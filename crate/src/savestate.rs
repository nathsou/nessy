const NESSIE: &[u8; 6] = b"NESSIE";
const HASH_SIZE: usize = 32; // bytes
const NESSIE_SAVE_VERSION: u8 = 0;
const VERSION_SIZE: usize = 1; // bytes
const HEADER_SIZE: usize = NESSIE.len() + VERSION_SIZE + HASH_SIZE; // bytes

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

    pub fn decode(buffer: &[u8]) -> Section {
        let name_len = buffer
            .iter()
            .position(|&c| c == b'\0')
            .expect("Could not read section name");

        let name = String::from_utf8_lossy(&buffer[..name_len]).to_string();

        let mut offset = name_len + 1;

        let data_size = u32::from_le_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]);

        offset += 4;

        let children_len = buffer[offset];
        offset += 1;

        let data = ByteBuffer::from(&buffer[offset..(offset + data_size as usize)]);
        offset += data_size as usize;

        let mut children = Vec::with_capacity(children_len as usize);

        for _ in 0..children_len {
            let child = Section::decode(&buffer[offset..]);
            offset += child.size();
            children.push(child);
        }

        Section {
            name,
            data,
            children,
        }
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
        self.name.as_bytes().len() + 1 // name + null terminator
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
        header[..NESSIE.len()].copy_from_slice(NESSIE);
        header[NESSIE.len()] = NESSIE_SAVE_VERSION;
        header[NESSIE.len() + VERSION_SIZE..].copy_from_slice(cart_rom_hash);

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
        header.copy_from_slice(&data[..HEADER_SIZE]);
        let mut offset = 0;

        let magic_number = &header[..NESSIE.len()];
        offset += NESSIE.len();

        if magic_number != NESSIE {
            return Err(SaveStateError::InvalidHeader);
        }

        let version = header[offset];
        offset += 1;

        if version > NESSIE_SAVE_VERSION {
            return Err(SaveStateError::InvalidVersion(version));
        }

        offset += HASH_SIZE;

        let root = Section::decode(&data[offset..]);

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

    pub fn write_slice(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    pub fn write_bool(&mut self, data: bool) {
        self.write_u8(data.into());
    }

    pub fn write_u8(&mut self, data: u8) {
        self.data.push(data)
    }

    pub fn write_u16(&mut self, data: u16) {
        self.data.push((data >> 8) as u8);
        self.data.push((data & 0xff) as u8);
    }

    pub fn write_u32(&mut self, data: u32) {
        self.write_u16((data >> 16) as u16);
        self.write_u16((data & 0xffff) as u16);
    }

    pub fn write_u64(&mut self, data: u64) {
        self.write_u32((data >> 32) as u32);
        self.write_u32((data & 0xffff_ffff) as u32);
    }

    pub fn read_slice(&mut self, dst: &mut [u8]) -> Result<(), SaveStateError> {
        if self.read_index + dst.len() > self.data.len() {
            return Err(SaveStateError::InvalidData);
        }

        let slice = &self.data[self.read_index..self.read_index + dst.len()];
        dst.copy_from_slice(slice);
        self.read_index += dst.len();

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
        self.read_u8().map(|val| val != 0)
    }

    pub fn read_u16(&mut self) -> Result<u16, SaveStateError> {
        let hi = self.read_u8()? as u16;
        let lo = self.read_u8()? as u16;

        Ok(hi << 8 | lo)
    }

    pub fn read_u32(&mut self) -> Result<u32, SaveStateError> {
        let hi = self.read_u16()? as u32;
        let lo = self.read_u16()? as u32;

        Ok(hi << 16 | lo)
    }

    pub fn read_u64(&mut self) -> Result<u64, SaveStateError> {
        let hi = self.read_u32()? as u64;
        let lo = self.read_u32()? as u64;

        Ok(hi << 32 | lo)
    }
}
