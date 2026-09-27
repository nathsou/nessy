const NESSY: &[u8; 5] = b"NESSY";
const HASH_SIZE: usize = 32; // bytes
const SAVE_VERSION: u8 = 1;
const VERSION_SIZE: usize = 1; // bytes
const HEADER_SIZE: usize = NESSY.len() + VERSION_SIZE + HASH_SIZE; // bytes

pub trait Save {
    fn save(&self, parent: &mut Section);
    fn load(&mut self, parent: &mut Section) -> Result<(), SaveStateError>;
}

pub struct Section {
    pub name: String,
    pub data: ByteBuffer,
    pub children: Vec<Section>,
}

impl Section {
    pub fn new(name: &str) -> Section {
        assert!(
            name.chars().all(|c| c != '\0'),
            "Section name cannot contain null bytes"
        );

        Section {
            name: name.into(),
            data: ByteBuffer::new(),
            children: vec![],
        }
    }

    pub fn encode_into(&self, buffer: &mut Vec<u8>) {
        // header: name, data size, number of children
        buffer.extend_from_slice(self.name.as_bytes());
        buffer.push(b'\0'); // null terminator
        buffer.extend_from_slice(&(self.data.size() as u32).to_le_bytes());
        buffer.push(self.children.len() as u8);

        buffer.extend_from_slice(self.data.get_data());

        for child in &self.children {
            child.encode_into(buffer);
        }
    }

    pub fn decode(buffer: &[u8]) -> Result<Section, SaveStateError> {
        let (section, used) = Self::decode_at(buffer, 0)?;
        if used != buffer.len() {
            return Err(SaveStateError::InvalidData);
        }
        Ok(section)
    }
    fn decode_at(buffer: &[u8], depth: usize) -> Result<(Section, usize), SaveStateError> {
        if depth > 16 {
            return Err(SaveStateError::InvalidData);
        }
        let name_len = buffer
            .iter()
            .take(256)
            .position(|b| *b == 0)
            .ok_or(SaveStateError::InvalidData)?;
        let name = std::str::from_utf8(&buffer[..name_len])
            .map_err(|_| SaveStateError::InvalidData)?
            .to_owned();
        let mut offset = name_len + 1;
        let header = buffer
            .get(offset..offset + 5)
            .ok_or(SaveStateError::InvalidData)?;
        let size = u32::from_le_bytes(header[..4].try_into().unwrap()) as usize;
        let child_count = header[4] as usize;
        offset += 5;
        let end = offset
            .checked_add(size)
            .ok_or(SaveStateError::InvalidData)?;
        let data = ByteBuffer::from(buffer.get(offset..end).ok_or(SaveStateError::InvalidData)?);
        offset = end;
        let mut children: Vec<Section> = Vec::new();
        for _ in 0..child_count {
            let (child, used) = Self::decode_at(
                buffer.get(offset..).ok_or(SaveStateError::InvalidData)?,
                depth + 1,
            )?;
            if children.iter().any(|s| s.name == child.name) {
                return Err(SaveStateError::InvalidData);
            }
            offset += used;
            children.push(child);
        }
        Ok((
            Section {
                name,
                data,
                children,
            },
            offset,
        ))
    }

    pub fn add_child(&mut self, child: Section) {
        self.children.push(child);
    }

    pub fn create_child(&mut self, name: &str) -> &mut Section {
        let child = Section::new(name);
        self.add_child(child);
        self.children.last_mut().unwrap()
    }

    pub fn get(&mut self, name: &str) -> Result<&mut Section, SaveStateError> {
        match self.get_child_aux(name) {
            Some(section) => Ok(section),
            None => Err(SaveStateError::MissingSection(name.to_owned())),
        }
    }

    fn get_child_aux(&mut self, name: &str) -> Option<&mut Section> {
        if self.name == name {
            return Some(self);
        }

        for child in &mut self.children {
            if let Some(section) = child.get_child_aux(name) {
                return Some(section);
            }
        }

        None
    }

    pub fn write_all(&mut self, values: &[impl Save]) {
        for value in values {
            value.save(self);
        }
    }

    pub fn read_all(&mut self, values: &mut [impl Save]) -> Result<(), SaveStateError> {
        for value in values {
            value.load(self)?;
        }

        Ok(())
    }

    /// in bytes
    pub fn size(&self) -> usize {
        self.name.len() + 1 // name + null terminator
        + 4 // data size
        + 1 // number of children
            + self.data.size()
            + self
                .children
                .iter()
                .map(|child| child.size())
                .sum::<usize>()
    }
}

#[derive(Debug)]
pub enum SaveStateError {
    InvalidHeader,
    InvalidVersion(u8),
    IncoherentRomHash {
        save_state_rom_hash: [u8; HASH_SIZE],
        cart_rom_hash: [u8; HASH_SIZE],
    },
    MissingSection(String),
    InvalidData,
}

pub struct SaveState {
    header: [u8; HEADER_SIZE],
    root: Section,
}

impl SaveState {
    #[allow(clippy::new_without_default)]
    pub fn new(cart_rom_hash: &[u8; HASH_SIZE]) -> Self {
        let mut header = [0; HEADER_SIZE];
        header[..NESSY.len()].copy_from_slice(NESSY);
        header[NESSY.len()] = SAVE_VERSION;
        header[NESSY.len() + VERSION_SIZE..].copy_from_slice(cart_rom_hash);

        SaveState {
            header,
            root: Section::new("root"),
        }
    }

    pub fn get_root_mut(&mut self) -> &mut Section {
        &mut self.root
    }

    pub fn decode(data: &[u8]) -> Result<SaveState, SaveStateError> {
        let mut header = [0; HEADER_SIZE];
        header.copy_from_slice(
            data.get(..HEADER_SIZE)
                .ok_or(SaveStateError::InvalidHeader)?,
        );
        let mut offset = 0;

        let magic_number = &header[..NESSY.len()];
        offset += NESSY.len();

        if magic_number != NESSY {
            return Err(SaveStateError::InvalidHeader);
        }

        let version = header[offset];
        offset += 1;

        if version != SAVE_VERSION {
            return Err(SaveStateError::InvalidVersion(version));
        }

        offset += HASH_SIZE;

        let root = Section::decode(&data[offset..])?;

        Ok(SaveState { header, root })
    }

    pub fn get_rom_hash(&self) -> [u8; HASH_SIZE] {
        let mut hash = [0; HASH_SIZE];
        hash.copy_from_slice(&self.header[HEADER_SIZE - HASH_SIZE..]);
        hash
    }

    pub fn encode(self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(HEADER_SIZE + self.root.size());
        buffer.extend_from_slice(&self.header);
        self.root.encode_into(&mut buffer);

        buffer
    }
}

pub struct ByteBuffer {
    data: Vec<u8>,
    read_index: usize,
}

impl ByteBuffer {
    pub fn new() -> Self {
        ByteBuffer {
            data: vec![],
            read_index: 0,
        }
    }

    pub fn from(data: &[u8]) -> Self {
        ByteBuffer {
            data: data.to_vec(),
            read_index: 0,
        }
    }

    /// in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn write_u8_slice(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    pub fn write_u32_slice(&mut self, data: &[u32]) {
        for value in data {
            self.write_u32(*value);
        }
    }

    pub fn write_bool(&mut self, data: bool) {
        self.write_u8(data.into());
    }

    pub fn write_u8(&mut self, data: u8) {
        self.data.push(data)
    }

    pub fn write_u16(&mut self, data: u16) {
        self.write_u8_slice(&data.to_le_bytes());
    }

    pub fn write_u32(&mut self, data: u32) {
        self.write_u8_slice(&data.to_le_bytes());
    }

    pub fn write_u64(&mut self, data: u64) {
        self.write_u8_slice(&data.to_le_bytes());
    }

    pub fn read_u8_slice(&mut self, dst: &mut [u8]) -> Result<(), SaveStateError> {
        if self.read_index + dst.len() > self.data.len() {
            return Err(SaveStateError::InvalidData);
        }

        let slice = &self.data[self.read_index..self.read_index + dst.len()];
        dst.copy_from_slice(slice);
        self.read_index += dst.len();

        Ok(())
    }

    pub fn read_u32_slice(&mut self, dst: &mut [u32]) -> Result<(), SaveStateError> {
        for value in dst {
            *value = self.read_u32()?;
        }

        Ok(())
    }

    pub fn read_u8(&mut self) -> Result<u8, SaveStateError> {
        if self.read_index >= self.data.len() {
            return Err(SaveStateError::InvalidData);
        }

        let value = self.data[self.read_index];
        self.read_index += 1;

        Ok(value)
    }

    pub fn read_bool(&mut self) -> Result<bool, SaveStateError> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(SaveStateError::InvalidData),
        }
    }

    pub fn read_u16(&mut self) -> Result<u16, SaveStateError> {
        let mut buf = [0; 2];
        self.read_u8_slice(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub fn read_u32(&mut self) -> Result<u32, SaveStateError> {
        let mut buf = [0; 4];
        self.read_u8_slice(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_u64(&mut self) -> Result<u64, SaveStateError> {
        let mut buf = [0; 8];
        self.read_u8_slice(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }
}

// Explicit field serialization keeps host layout/padding out of the portable state format.
pub(crate) trait StateValue {
    fn put(&self, data: &mut ByteBuffer);
    fn get(&mut self, data: &mut ByteBuffer) -> Result<(), SaveStateError>;
}
macro_rules! integer_value {
    ($ty:ty,$write:ident,$read:ident) => {
        impl StateValue for $ty {
            fn put(&self, data: &mut ByteBuffer) {
                data.$write(*self);
            }
            fn get(&mut self, data: &mut ByteBuffer) -> Result<(), SaveStateError> {
                *self = data.$read()?;
                Ok(())
            }
        }
    };
}
integer_value!(u8, write_u8, read_u8);
integer_value!(u16, write_u16, read_u16);
integer_value!(u32, write_u32, read_u32);
integer_value!(u64, write_u64, read_u64);
integer_value!(bool, write_bool, read_bool);
impl StateValue for f32 {
    fn put(&self, data: &mut ByteBuffer) {
        data.write_u32(self.to_bits());
    }
    fn get(&mut self, data: &mut ByteBuffer) -> Result<(), SaveStateError> {
        *self = f32::from_bits(data.read_u32()?);
        if self.is_finite() {
            Ok(())
        } else {
            Err(SaveStateError::InvalidData)
        }
    }
}
impl StateValue for f64 {
    fn put(&self, data: &mut ByteBuffer) {
        data.write_u64(self.to_bits());
    }
    fn get(&mut self, data: &mut ByteBuffer) -> Result<(), SaveStateError> {
        *self = f64::from_bits(data.read_u64()?);
        if self.is_finite() {
            Ok(())
        } else {
            Err(SaveStateError::InvalidData)
        }
    }
}
impl<T: StateValue, const N: usize> StateValue for [T; N] {
    fn put(&self, data: &mut ByteBuffer) {
        for v in self {
            v.put(data);
        }
    }
    fn get(&mut self, data: &mut ByteBuffer) -> Result<(), SaveStateError> {
        for v in self {
            v.get(data)?;
        }
        Ok(())
    }
}
impl<T: StateValue> StateValue for Box<T> {
    fn put(&self, data: &mut ByteBuffer) {
        (**self).put(data);
    }
    fn get(&mut self, data: &mut ByteBuffer) -> Result<(), SaveStateError> {
        (**self).get(data)
    }
}
impl<T: StateValue + Default> StateValue for Option<T> {
    fn put(&self, data: &mut ByteBuffer) {
        data.write_bool(self.is_some());
        if let Some(v) = self {
            v.put(data);
        }
    }
    fn get(&mut self, data: &mut ByteBuffer) -> Result<(), SaveStateError> {
        *self = if data.read_bool()? {
            let mut v = T::default();
            v.get(data)?;
            Some(v)
        } else {
            None
        };
        Ok(())
    }
}
macro_rules! state_fields {
    ($ty:ty, $($field:ident),+ $(,)?)=>{
        impl crate::savestate::StateValue for $ty {
            fn put(&self,data:&mut crate::savestate::ByteBuffer){$(crate::savestate::StateValue::put(&self.$field,data);)+}
            fn get(&mut self,data:&mut crate::savestate::ByteBuffer)->Result<(),crate::savestate::SaveStateError>{$(crate::savestate::StateValue::get(&mut self.$field,data)?;)+Ok(())}
        }
    }
}
pub(crate) use state_fields;

impl Default for ByteBuffer {
    fn default() -> Self {
        Self::new()
    }
}
