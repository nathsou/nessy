(function(){const t=document.createElement("link").relList;if(t&&t.supports&&t.supports("modulepreload"))return;for(const s of document.querySelectorAll('link[rel="modulepreload"]'))r(s);new MutationObserver(s=>{for(const a of s)if(a.type==="childList")for(const c of a.addedNodes)c.tagName==="LINK"&&c.rel==="modulepreload"&&r(c)}).observe(document,{childList:!0,subtree:!0});function n(s){const a={};return s.integrity&&(a.integrity=s.integrity),s.referrerPolicy&&(a.referrerPolicy=s.referrerPolicy),s.crossOrigin==="use-credentials"?a.credentials="include":s.crossOrigin==="anonymous"?a.credentials="omit":a.credentials="same-origin",a}function r(s){if(s.ep)return;s.ep=!0;const a=n(s);fetch(s.href,a)}})();let _,B=null;function O(){return(B===null||B.byteLength===0)&&(B=new Uint8Array(_.memory.buffer)),B}function de(e,t){return e=e>>>0,O().subarray(e/1,e/1+t)}const I=new Array(128).fill(void 0);I.push(void 0,null,!0,!1);function N(e){return I[e]}let W=I.length;function xe(e){e<132||(I[e]=W,W=e)}function Q(e){const t=N(e);return xe(e),t}const fe=typeof TextDecoder<"u"?new TextDecoder("utf-8",{ignoreBOM:!0,fatal:!0}):{decode:()=>{throw Error("TextDecoder not available")}};typeof TextDecoder<"u"&&fe.decode();function Z(e,t){return e=e>>>0,fe.decode(O().subarray(e,e+t))}function U(e){W===I.length&&I.push(I.length+1);const t=W;return W=I[t],I[t]=e,t}let E=0;function G(e,t){const n=t(e.length*1,1)>>>0;return O().set(e,n/1),E=e.length,n}let H=null;function T(){return(H===null||H.byteLength===0)&&(H=new Int32Array(_.memory.buffer)),H}let j=null;function Ee(){return(j===null||j.byteLength===0)&&(j=new Float32Array(_.memory.buffer)),j}function re(e,t){const n=t(e.length*4,4)>>>0;return Ee().set(e,n/4),E=e.length,n}const X=typeof TextEncoder<"u"?new TextEncoder("utf-8"):{encode:()=>{throw Error("TextEncoder not available")}},Ae=typeof X.encodeInto=="function"?function(e,t){return X.encodeInto(e,t)}:function(e,t){const n=X.encode(e);return t.set(n),{read:e.length,written:n.length}};function Te(e,t,n){if(n===void 0){const i=X.encode(e),u=t(i.length,1)>>>0;return O().subarray(u,u+i.length).set(i),E=i.length,u}let r=e.length,s=t(r,1)>>>0;const a=O();let c=0;for(;c<r;c++){const i=e.charCodeAt(c);if(i>127)break;a[s+c]=i}if(c!==r){c!==0&&(e=e.slice(c)),s=n(s,r,r=c+e.length*3,1)>>>0;const i=O().subarray(s+c,s+r),u=Ae(e,i);c+=u.written}return E=c,s}class F{static __wrap(t){t=t>>>0;const n=Object.create(F.prototype);return n.__wbg_ptr=t,n}__destroy_into_raw(){const t=this.__wbg_ptr;return this.__wbg_ptr=0,t}free(){const t=this.__destroy_into_raw();_.__wbg_nes_free(t)}static initPanicHook(){_.nes_initPanicHook()}static new(t,n){try{const c=_.__wbindgen_add_to_stack_pointer(-16),i=G(t,_.__wbindgen_malloc),u=E;_.nes_new(c,i,u,n);var r=T()[c/4+0],s=T()[c/4+1],a=T()[c/4+2];if(a)throw Q(s);return F.__wrap(r)}finally{_.__wbindgen_add_to_stack_pointer(16)}}softReset(){_.nes_softReset(this.__wbg_ptr)}nextFrame(t){var n=G(t,_.__wbindgen_malloc),r=E;_.nes_nextFrame(this.__wbg_ptr,n,r,U(t))}nextSamples(t){var n=re(t,_.__wbindgen_malloc),r=E;return _.nes_nextSamples(this.__wbg_ptr,n,r,U(t))!==0}fillFrameBuffer(t){var n=G(t,_.__wbindgen_malloc),r=E;_.nes_fillFrameBuffer(this.__wbg_ptr,n,r,U(t))}setJoypad1(t){_.nes_setJoypad1(this.__wbg_ptr,t)}saveState(){try{const s=_.__wbindgen_add_to_stack_pointer(-16);_.nes_saveState(s,this.__wbg_ptr);var t=T()[s/4+0],n=T()[s/4+1],r=de(t,n).slice();return _.__wbindgen_free(t,n*1),r}finally{_.__wbindgen_add_to_stack_pointer(16)}}loadState(t){try{const s=_.__wbindgen_add_to_stack_pointer(-16),a=G(t,_.__wbindgen_malloc),c=E;_.nes_loadState(s,this.__wbg_ptr,a,c);var n=T()[s/4+0],r=T()[s/4+1];if(r)throw Q(n)}finally{_.__wbindgen_add_to_stack_pointer(16)}}fillAudioBuffer(t,n){var r=re(t,_.__wbindgen_malloc),s=E;_.nes_fillAudioBuffer(this.__wbg_ptr,r,s,U(t),n)}}async function Ie(e,t){if(typeof Response=="function"&&e instanceof Response){if(typeof WebAssembly.instantiateStreaming=="function")try{return await WebAssembly.instantiateStreaming(e,t)}catch(r){if(e.headers.get("Content-Type")!="application/wasm")console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n",r);else throw r}const n=await e.arrayBuffer();return await WebAssembly.instantiate(n,t)}else{const n=await WebAssembly.instantiate(e,t);return n instanceof WebAssembly.Instance?{instance:n,module:e}:n}}function Re(){const e={};return e.wbg={},e.wbg.__wbindgen_copy_to_typed_array=function(t,n,r){new Uint8Array(N(r).buffer,N(r).byteOffset,N(r).byteLength).set(de(t,n))},e.wbg.__wbindgen_object_drop_ref=function(t){Q(t)},e.wbg.__wbindgen_string_new=function(t,n){const r=Z(t,n);return U(r)},e.wbg.__wbg_new_abda76e883ba8a5f=function(){const t=new Error;return U(t)},e.wbg.__wbg_stack_658279fe44541cf6=function(t,n){const r=N(n).stack,s=Te(r,_.__wbindgen_malloc,_.__wbindgen_realloc),a=E;T()[t/4+1]=a,T()[t/4+0]=s},e.wbg.__wbg_error_f851667af71bcfc6=function(t,n){let r,s;try{r=t,s=n,console.error(Z(t,n))}finally{_.__wbindgen_free(r,s,1)}},e.wbg.__wbindgen_throw=function(t,n){throw new Error(Z(t,n))},e}function Le(e,t){return _=e.exports,me.__wbindgen_wasm_module=t,j=null,H=null,B=null,_}async function me(e){if(_!==void 0)return _;typeof e>"u"&&(e=new URL("/nessy/wasm/nessy_bg.wasm",self.location));const t=Re();(typeof e=="string"||typeof Request=="function"&&e instanceof Request||typeof URL=="function"&&e instanceof URL)&&(e=fetch(e));const{instance:n,module:r}=await Ie(await e,t);return Le(n,r)}const Pe=()=>{const e={};return{register(t,n){if(e[t]!=null)throw new Error(`Hook ${String(t)} is already registered`);e[t]=n},call(t,...n){const r=e[t];if(r!=null)return r.apply(null,n);throw new Error(`Hook ${String(t)} is not registered`)}}},y=Pe();var S=(e=>(e[e.A=1]="A",e[e.B=2]="B",e[e.SELECT=4]="SELECT",e[e.START=8]="START",e[e.UP=16]="UP",e[e.DOWN=32]="DOWN",e[e.LEFT=64]="LEFT",e[e.RIGHT=128]="RIGHT",e))(S||{});const se={up:16,left:64,down:32,right:128,b:2,a:1,start:8,select:4},ke={up:"w",left:"a",down:"s",right:"d",b:"k",a:"l",start:"Enter",select:" "},ge=(e=ke)=>{const t={},n=()=>JSON.stringify(e),r=s=>{for(const[a,c]of Object.entries(s))s[a]=c,t[c]=se[a]};return r(e),{ref:e,set(s,a){delete t[e[a]],t[s]=se[a],e[a]=s},get(s){return t[s]??null},isKeyMapped(s){return s in t},serialize:n,update:r}},De=e=>{let t=0,n=0,r=!1;const s=[];let a=!1;async function c(o,f){if(o.key==="Meta"&&(a=f,o.preventDefault()),a&&f)switch(o.key){case"s":{o.preventDefault(),y.call("saveState");return}case"r":{o.preventDefault(),y.call("softReset");return}case"l":{o.preventDefault(),await y.call("loadLastSave");return}default:o.preventDefault()}if(e.ref.controls.isKeyMapped(o.key)){(o.key==="Enter"||o.key===" ")&&o.preventDefault();const g=n,m=e.ref.controls.get(o.key);f?n|=m:n&=~m,g!==n&&(r=!0),y.call("setJoypad1",n)}}const i=o=>c(o,!0),u=o=>c(o,!1);function l(){t+=1,r&&(s.push(t,n),r=!1)}function d(){}return{onKeyDown:i,onKeyUp:u,history:s,tick:l,save:d}};function Me(e){const t=e.getContext("webgl");if(t==null)throw new Error("Unable to get WebGL context. Your browser may not support it.");const n=t.createTexture();if(n==null)throw new Error("Unable to create texture.");t.bindTexture(t.TEXTURE_2D,n),t.texParameteri(t.TEXTURE_2D,t.TEXTURE_WRAP_S,t.CLAMP_TO_EDGE),t.texParameteri(t.TEXTURE_2D,t.TEXTURE_WRAP_T,t.CLAMP_TO_EDGE),t.texParameteri(t.TEXTURE_2D,t.TEXTURE_MIN_FILTER,t.NEAREST),t.texParameteri(t.TEXTURE_2D,t.TEXTURE_MAG_FILTER,t.NEAREST);const r=t.createBuffer();if(r==null)throw new Error("Unable to create buffer.");t.bindBuffer(t.ARRAY_BUFFER,r),t.bufferData(t.ARRAY_BUFFER,new Float32Array([-1,-1,1,-1,-1,1,-1,1,1,-1,1,1]),t.STATIC_DRAW);const s=`
        attribute vec2 a_position;
        varying vec2 v_texCoord;

        void main() {
            gl_Position = vec4(a_position, 0, 1);
            v_texCoord = a_position * vec2(0.5, -0.5) + 0.5;
        }
    `,a=`
        precision mediump float;
        uniform sampler2D u_image;
        varying vec2 v_texCoord;

        void main() {
            gl_FragColor = texture2D(u_image, v_texCoord);
        }
    `,c=d(t.VERTEX_SHADER,s),i=d(t.FRAGMENT_SHADER,a),u=o(c,i),l=t.getAttribLocation(u,"a_position");t.viewport(0,0,256,240);function d(g,m){const p=t.createShader(g);if(!p)throw new Error("Unable to create shader.");if(t.shaderSource(p,m),t.compileShader(p),!t.getShaderParameter(p,t.COMPILE_STATUS))throw`Could not compile WebGL shader. 

`+t.getShaderInfoLog(p);return p}function o(g,m){const p=t.createProgram();if(!p)throw new Error("Unable to create program.");if(t.attachShader(p,g),t.attachShader(p,m),t.linkProgram(p),!t.getProgramParameter(p,t.LINK_STATUS))throw`Could not compile WebGL program. 

`+t.getProgramInfoLog(p);return p}function f(g){t.useProgram(u),t.enableVertexAttribArray(l),t.bindBuffer(t.ARRAY_BUFFER,r),t.vertexAttribPointer(l,2,t.FLOAT,!1,0,0),t.bindTexture(t.TEXTURE_2D,n),t.texImage2D(t.TEXTURE_2D,0,t.RGB,e.width,e.height,0,t.RGB,t.UNSIGNED_BYTE,g),t.drawArrays(t.TRIANGLES,0,6)}return{render:f}}const Ue=()=>{let e=0;const t=new Map,n={saved:[],uiToggled:[]};return{emit:(c,i)=>{n[c].forEach(({handler:u})=>u(i))},on:(c,i)=>(e+=1,n[c].push({id:e,handler:i}),t.set(e,c),e),remove:c=>{const i=t.get(c);if(i!=null){t.delete(c);const u=n[i].findIndex(l=>l.id===c);u!==-1&&n[i].splice(u,1)}}}},V=Ue(),Oe=()=>[],q=e=>e,ae="nessy.store",Fe=()=>({version:1,frameCount:0,rom:q(null),controls:ge(),scalingFactor:q(4),scalingMode:q("pixelated"),lastState:q(null)}),J={serialize(e){return Array.from(e).map(t=>String.fromCharCode(t)).join("")},deserialize(e){return Uint8Array.from(e.split("").map(t=>t.charCodeAt(0)))},async hash(e){const t=await crypto.subtle.digest("SHA-256",e);return Array.from(new Uint8Array(t)).map(n=>n.toString(16).padStart(2,"0")).join("")}},Ce=async()=>{const e=await new Promise((o,f)=>{const g=indexedDB.open("nessy",1);g.onerror=f,g.onupgradeneeded=()=>{const m=g.result,p=m.createObjectStore("roms",{keyPath:"hash"});p.createIndex("hash","hash",{unique:!0}),p.createIndex("name","name",{unique:!1}),p.createIndex("data","data",{unique:!1});const h=m.createObjectStore("saves",{keyPath:"timestamp"});h.createIndex("timestamp","timestamp",{unique:!0}),h.createIndex("romHash","romHash",{unique:!1}),h.createIndex("state","state",{unique:!1});const v=m.createObjectStore("titleScreens",{keyPath:"romHash"});v.createIndex("romHash","romHash",{unique:!0}),v.createIndex("data","data",{unique:!1})},g.onsuccess=()=>{o(g.result)}});async function t(o,f){const g={hash:await J.hash(f),name:o.endsWith(".nes")?o.slice(0,-4):o,data:f},h=e.transaction(["roms"],"readwrite").objectStore("roms").put(g);return new Promise((v,x)=>{h.onerror=x,h.onsuccess=()=>{v(g.hash)}})}async function n(o){return new Promise((f,g)=>{const h=e.transaction(["roms"],"readonly").objectStore("roms").get(o);h.onerror=g,h.onsuccess=()=>{h.result==null?g(new Error(`ROM with hash ${o} not found`)):f(h.result)}})}async function r(){return new Promise((o,f)=>{const p=e.transaction(["roms"],"readonly").objectStore("roms").getAll();p.onerror=()=>{o([])},p.onsuccess=()=>{o(p.result)}})}async function s(o,f){return new Promise((g,m)=>{const p={timestamp:Date.now(),romHash:o,state:f},x=e.transaction(["saves"],"readwrite").objectStore("saves").put(p);x.onerror=m,x.onsuccess=()=>{g(p.timestamp)}})}async function a(o){return new Promise((f,g)=>{const h=e.transaction(["saves"],"readonly").objectStore("saves").get(o);h.onerror=g,h.onsuccess=()=>{h.result==null?g(new Error(`Save with timestamp ${o} not found`)):f(h.result)}})}async function c(o){return new Promise((f,g)=>{const v=e.transaction(["saves"],"readonly").objectStore("saves").index("romHash").openCursor(IDBKeyRange.only(o),"prev");v.onerror=g,v.onsuccess=()=>{v.result==null?f(null):f(v.result.value)}})}async function i(o){return new Promise((f,g)=>{const v=e.transaction(["saves"],"readonly").objectStore("saves").index("romHash").getAll(o);v.onerror=()=>{f([])},v.onsuccess=()=>{f(v.result)}})}async function u(o,f){return new Promise((g,m)=>{const p={romHash:o,data:f},x=e.transaction(["titleScreens"],"readwrite").objectStore("titleScreens").put(p);x.onerror=m,x.onsuccess=()=>{g()}})}async function l(o){return new Promise((f,g)=>{const h=e.transaction(["titleScreens"],"readonly").objectStore("titleScreens").get(o);h.onerror=g,h.onsuccess=()=>{h.result==null?f(null):f(h.result)}})}async function d(){return new Promise((o,f)=>{const p=e.transaction(["titleScreens"],"readonly").objectStore("titleScreens").getAll();p.onerror=()=>{o([])},p.onsuccess=()=>{o(p.result)}})}return{rom:{get:n,insert:t,list:r},save:{get:a,getLast:c,insert:s,list:i},titleScreen:{get:l,insert:u,list:d}}},Be=async()=>{const e=await Ce(),t=l=>JSON.stringify({...l,controls:l.controls.ref,lastState:l.lastState!=null?J.serialize(l.lastState):null}),n=l=>{const d=JSON.parse(l),o=ge();return o.update(d.controls),d.controls=o,d.lastState=d.lastState!=null?J.deserialize(d.lastState):null,d},r=(()=>{const l=localStorage.getItem(ae);return l!=null?n(l):Fe()})(),s=Oe();return{ref:r,subscribe:(l,d)=>{s.push({key:l,handler:d})},get:l=>r[l],set:(l,d)=>{const o=r[l];r[l]=d,s.forEach(({key:f,handler:g})=>{f===l&&g(d,o)})},save:()=>{localStorage.setItem(ae,t(r))},db:e}},P=32,Ne=e=>{let t=0;function n(s){e=s,t=Math.floor((P-s.width)/2),r.height=s.height}const r={state:{},width:P,height:e.height,render(s,a,c){e.render(s+t,a,c)},update:n};return n(e),r},He=new Uint8Array([0,0,0,0,0,0,0,0,126,129,165,129,157,185,129,126,126,255,219,255,227,199,255,126,108,254,254,254,124,56,16,0,16,56,124,254,124,56,16,0,56,124,56,254,254,16,16,124,0,24,60,126,255,126,24,126,0,0,24,60,60,24,0,0,255,255,231,195,195,231,255,255,0,60,102,66,66,102,60,0,255,195,153,189,189,153,195,255,15,7,15,125,204,204,204,120,60,102,102,102,60,24,126,24,63,51,63,48,48,112,240,224,127,99,127,99,99,103,230,192,153,90,60,231,231,60,90,153,128,224,248,254,248,224,128,0,2,14,62,254,62,14,2,0,24,60,126,24,24,126,60,24,102,102,102,102,102,0,102,0,127,219,219,123,27,27,27,0,63,96,124,102,102,62,6,252,0,0,0,0,126,126,126,0,24,60,126,24,126,60,24,255,24,60,126,24,24,24,24,0,24,24,24,24,126,60,24,0,0,24,12,254,12,24,0,0,0,48,96,254,96,48,0,0,0,0,192,192,192,254,0,0,0,36,102,255,102,36,0,0,0,24,60,126,255,255,0,0,0,255,255,126,60,24,0,0,0,0,0,0,0,0,0,0,24,24,24,24,24,0,24,0,108,108,108,0,0,0,0,0,108,108,254,108,254,108,108,0,24,126,192,124,6,252,24,0,0,198,204,24,48,102,198,0,56,108,56,118,220,204,118,0,48,48,96,0,0,0,0,0,12,24,48,48,48,24,12,0,48,24,12,12,12,24,48,0,0,102,60,255,60,102,0,0,0,24,24,126,24,24,0,0,0,0,0,0,0,24,24,48,0,0,0,126,0,0,0,0,0,0,0,0,0,24,24,0,6,12,24,48,96,192,128,0,124,206,222,246,230,198,124,0,24,56,24,24,24,24,126,0,124,198,6,124,192,192,254,0,252,6,6,60,6,6,252,0,12,204,204,204,254,12,12,0,254,192,252,6,6,198,124,0,124,192,192,252,198,198,124,0,254,6,6,12,24,48,48,0,124,198,198,124,198,198,124,0,124,198,198,126,6,6,124,0,0,24,24,0,0,24,24,0,0,24,24,0,0,24,24,48,12,24,48,96,48,24,12,0,0,0,126,0,126,0,0,0,48,24,12,6,12,24,48,0,60,102,12,24,24,0,24,0,124,198,222,222,222,192,126,0,56,108,198,198,254,198,198,0,252,198,198,252,198,198,252,0,124,198,192,192,192,198,124,0,248,204,198,198,198,204,248,0,254,192,192,248,192,192,254,0,254,192,192,248,192,192,192,0,124,198,192,192,206,198,124,0,198,198,198,254,198,198,198,0,126,24,24,24,24,24,126,0,6,6,6,6,6,198,124,0,198,204,216,240,216,204,198,0,192,192,192,192,192,192,254,0,198,238,254,254,214,198,198,0,198,230,246,222,206,198,198,0,124,198,198,198,198,198,124,0,252,198,198,252,192,192,192,0,124,198,198,198,214,222,124,6,252,198,198,252,216,204,198,0,124,198,192,124,6,198,124,0,255,24,24,24,24,24,24,0,198,198,198,198,198,198,254,0,198,198,198,198,198,124,56,0,198,198,198,198,214,254,108,0,198,198,108,56,108,198,198,0,198,198,198,124,24,48,224,0,254,6,12,24,48,96,254,0,60,48,48,48,48,48,60,0,192,96,48,24,12,6,2,0,60,12,12,12,12,12,60,0,16,56,108,198,0,0,0,0,0,0,0,0,0,0,0,255,24,24,12,0,0,0,0,0,0,0,124,6,126,198,126,0,192,192,192,252,198,198,252,0,0,0,124,198,192,198,124,0,6,6,6,126,198,198,126,0,0,0,124,198,254,192,124,0,28,54,48,120,48,48,120,0,0,0,126,198,198,126,6,252,192,192,252,198,198,198,198,0,24,0,56,24,24,24,60,0,6,0,6,6,6,6,198,124,192,192,204,216,248,204,198,0,56,24,24,24,24,24,60,0,0,0,204,254,254,214,214,0,0,0,252,198,198,198,198,0,0,0,124,198,198,198,124,0,0,0,252,198,198,252,192,192,0,0,126,198,198,126,6,6,0,0,252,198,192,192,192,0,0,0,126,192,124,6,252,0,24,24,126,24,24,24,14,0,0,0,198,198,198,198,126,0,0,0,198,198,198,124,56,0,0,0,198,198,214,254,108,0,0,0,198,108,56,108,198,0,0,0,198,198,198,126,6,252,0,0,254,12,56,96,254,0,14,24,24,112,24,24,14,0,24,24,24,0,24,24,24,0,112,24,24,14,24,24,112,0,118,220,0,0,0,0,0,0,0,16,56,108,198,198,254,0,124,198,192,192,192,214,124,48,198,0,198,198,198,198,126,0,14,0,124,198,254,192,124,0,126,129,60,6,126,198,126,0,102,0,124,6,126,198,126,0,224,0,124,6,126,198,126,0,24,24,124,6,126,198,126,0,0,0,124,198,192,214,124,48,126,129,124,198,254,192,124,0,102,0,124,198,254,192,124,0,224,0,124,198,254,192,124,0,102,0,56,24,24,24,60,0,124,130,56,24,24,24,60,0,112,0,56,24,24,24,60,0,198,16,124,198,254,198,198,0,56,56,0,124,198,254,198,0,14,0,254,192,248,192,254,0,0,0,127,12,127,204,127,0,63,108,204,255,204,204,207,0,124,130,124,198,198,198,124,0,102,0,124,198,198,198,124,0,224,0,124,198,198,198,124,0,124,130,0,198,198,198,126,0,224,0,198,198,198,198,126,0,102,0,102,102,102,62,6,124,198,124,198,198,198,198,124,0,198,0,198,198,198,198,254,0,24,24,126,216,216,216,126,24,56,108,96,240,96,102,252,0,102,102,60,24,126,24,126,24,248,204,204,250,198,207,198,195,14,27,24,60,24,24,216,112,14,0,124,6,126,198,126,0,28,0,56,24,24,24,60,0,14,0,124,198,198,198,124,0,14,0,198,198,198,198,126,0,0,254,0,252,198,198,198,0,254,0,198,230,246,222,206,0,60,108,108,62,0,126,0,0,60,102,102,60,0,126,0,0,24,0,24,24,48,102,60,0,0,0,0,252,192,192,0,0,0,0,0,252,12,12,0,0,198,204,216,63,99,207,140,15,195,198,204,219,55,109,207,3,24,0,24,24,24,24,24,0,0,51,102,204,102,51,0,0,0,204,102,51,102,204,0,0,34,136,34,136,34,136,34,136,85,170,85,170,85,170,85,170,221,119,221,119,221,119,221,119,24,24,24,24,24,24,24,24,24,24,24,24,248,24,24,24,24,24,248,24,248,24,24,24,54,54,54,54,246,54,54,54,0,0,0,0,254,54,54,54,0,0,248,24,248,24,24,24,54,54,246,6,246,54,54,54,54,54,54,54,54,54,54,54,0,0,254,6,246,54,54,54,54,54,246,6,254,0,0,0,54,54,54,54,254,0,0,0,24,24,248,24,248,0,0,0,0,0,0,0,248,24,24,24,24,24,24,24,31,0,0,0,24,24,24,24,255,0,0,0,0,0,0,0,255,24,24,24,24,24,24,24,31,24,24,24,0,0,0,0,255,0,0,0,24,24,24,24,255,24,24,24,24,24,31,24,31,24,24,24,54,54,54,54,55,54,54,54,54,54,55,48,63,0,0,0,0,0,63,48,55,54,54,54,54,54,247,0,255,0,0,0,0,0,255,0,247,54,54,54,54,54,55,48,55,54,54,54,0,0,255,0,255,0,0,0,54,54,247,0,247,54,54,54,24,24,255,0,255,0,0,0,54,54,54,54,255,0,0,0,0,0,255,0,255,24,24,24,0,0,0,0,255,54,54,54,54,54,54,54,63,0,0,0,24,24,31,24,31,0,0,0,0,0,31,24,31,24,24,24,0,0,0,0,63,54,54,54,54,54,54,54,255,54,54,54,24,24,255,24,255,24,24,24,24,24,24,24,248,0,0,0,0,0,0,0,31,24,24,24,255,255,255,255,255,255,255,255,0,0,0,0,255,255,255,255,240,240,240,240,240,240,240,240,15,15,15,15,15,15,15,15,255,255,255,255,0,0,0,0,0,0,118,220,200,220,118,0,56,108,108,120,108,102,108,96,0,254,198,192,192,192,192,0,0,0,254,108,108,108,108,0,254,96,48,24,48,96,254,0,0,0,126,216,216,216,112,0,0,102,102,102,102,124,96,192,0,118,220,24,24,24,24,0,126,24,60,102,102,60,24,126,60,102,195,255,195,102,60,0,60,102,195,195,102,102,231,0,14,24,12,126,198,198,124,0,0,0,126,219,219,126,0,0,6,12,126,219,219,126,96,192,56,96,192,248,192,96,56,0,120,204,204,204,204,204,204,0,0,126,0,126,0,126,0,0,24,24,126,24,24,0,126,0,96,48,24,48,96,0,252,0,24,48,96,48,24,0,252,0,14,27,27,24,24,24,24,24,24,24,24,24,24,216,216,112,24,24,0,126,0,24,24,0,0,118,220,0,118,220,0,0,56,108,108,56,0,0,0,0,0,0,0,24,24,0,0,0,0,0,0,0,24,0,0,0,15,12,12,12,236,108,60,28,120,108,108,108,108,0,0,0,124,12,124,96,124,0,0,0,0,0,60,60,60,60,0,0,0,16,0,0,0,0,0,0]);function je(e,t=63,n=48){const s=(e.charCodeAt(0)&255)*8,a=He.slice(s,s+8),c=new Uint8Array(64);for(let i=0;i<8;i++){const u=a[i];for(let l=0;l<8;l++)c[i*8+l]=u&1<<7-l?t:n}return c}function K(e,t,n,r,s=48,a=0){for(let c=0;c<n.length;c++)r.setTile(e+c,t,je(n[c],s,a))}const oe={textColor:48,bgColor:0,maxLength:1/0},ie=(e,t)=>e.length>t?e.slice(0,t-3)+"...":e,A=(e,t=oe)=>{const n={active:!1},r={...oe,...t};e=ie(e,r.maxLength);const s={state:n,width:e.length,height:1,update(a){a!==e&&(s.width=a.length,e=ie(a,r.maxLength??1/0))},render:(a,c,i)=>{n.active?K(a,c,e,i,r.bgColor,r.textColor):K(a,c,e,i,r.textColor,r.bgColor)}};return s},We=(e,t)=>{let n,r=!1;switch(e){case S.UP:n="up";break;case S.LEFT:n="left";break;case S.DOWN:n="down";break;case S.RIGHT:n="right";break;case S.A:n="a";break;case S.B:n="b";break;case S.START:n="start";break;case S.SELECT:n="select";break}const s=()=>{const c=`${S[e].padEnd(6," ")}`;if(r)return`${c} > ...`;let i=t.ref.controls.ref[n];return i===" "&&(i="space"),`${c} > ${i.toUpperCase()}`},a=A(s());return{...a,render(c,i,u){a.update(s()),a.render(c,i,u)},onKeyDown(c){return r?(t.ref.controls.set(c,n),r=!1,a.update(s()),!0):c==="Enter"?(r=!0,!0):!1}}},Ke=(e,t={spacing:1,align:"start",firstIndex:0,lastIndex:e.length-1})=>{const n={...t};let r=0,s=0;for(const a of e)r=Math.max(r,a.width),s+=a.height+n.spacing;return{state:n,width:r,height:s,render(a,c,i){let u=c;for(let l=n.firstIndex;l<=n.lastIndex;l++){const d=e[l];let o;switch(n.align){case"start":o=0;break;case"center":o=(r-d.width)/2;break;case"end":o=r-d.width;break}d.render(o+a,u,i),u+=d.height+n.spacing}},update(a){e=a,n.firstIndex=0,n.lastIndex=e.length-1}}},Y=(e,{visibleItems:t=e.length,onSelect:n}={})=>{const r={activeIndex:0,items:e},s=Ke(e);e[0].state.active=!0;const a=i=>{e[r.activeIndex].state.active=!1,r.activeIndex=i,e[i].state.active=!0,n==null||n(i)},c=()=>{const i=s.state;r.activeIndex<i.firstIndex&&(i.firstIndex=Math.max(r.activeIndex,0),i.lastIndex=Math.min(i.firstIndex+t-1,e.length-1)),r.activeIndex>i.lastIndex&&(i.firstIndex=Math.max(r.activeIndex-t+1,0),i.lastIndex=Math.min(r.activeIndex,e.length-1))};return c(),{...s,state:r,next(){a((r.activeIndex+1)%e.length),c()},prev(){a(Math.max(r.activeIndex===0?e.length-1:r.activeIndex-1,0)),c()},update(i){s.update(i),e=i,r.items=i,r.activeIndex=0,s.state.firstIndex=0,s.state.lastIndex=Math.min(t,i.length)-1}}},Ge=e=>{const t=[S.UP,S.LEFT,S.DOWN,S.RIGHT,S.A,S.B,S.START,S.SELECT].map(a=>We(a,e)),n=Y(t);return{...n,onKeyDown:a=>t[n.state.activeIndex].onKeyDown(a),setActive:a=>{}}},qe=(e,{initialIndex:t=0,onSelect:n}={})=>{const r={active:!0,activeIndex:t};e[t].state.active=!0;const s=[];let a=0,c=0;for(let u=0;u<e.length;u++){const l=e[u];s.push(Math.round(c+l.width/2)),c+=l.width+1,a=Math.max(a,l.width)}const i=u=>{u!==r.activeIndex&&(e[r.activeIndex].state.active=!1,e[u].state.active=!0,r.activeIndex=u,n==null||n(u))};return i(t),{state:r,width:P,height:1,render(u,l,d){let o=P/2-s[r.activeIndex];for(let f=0;f<e.length;f++){const g=e[f];g.render(o,l,d),o+=g.width+1}r.activeIndex>0&&K(0,l,"< ",d),r.activeIndex<e.length-1&&K(P-2,l," >",d)},next(){i(Math.min(r.activeIndex+1,e.length-1))},prev(){i(Math.max(r.activeIndex-1,0))}}},ee=(()=>{if(typeof self>"u")return!1;if("top"in self&&self!==top)try{}catch{return!1}return"showOpenFilePicker"in self})(),$e=ee?Promise.resolve().then(function(){return Ye}):Promise.resolve().then(function(){return nt});async function Xe(...e){return(await $e).default(...e)}ee?Promise.resolve().then(function(){return Qe}):Promise.resolve().then(function(){return st});ee?Promise.resolve().then(function(){return et}):Promise.resolve().then(function(){return ot});const ze=async e=>{const t=await e.getFile();return t.handle=e,t};var Ve=async(e=[{}])=>{Array.isArray(e)||(e=[e]);const t=[];e.forEach((s,a)=>{t[a]={description:s.description||"Files",accept:{}},s.mimeTypes?s.mimeTypes.map(c=>{t[a].accept[c]=s.extensions||[]}):t[a].accept["*/*"]=s.extensions||[]});const n=await window.showOpenFilePicker({id:e[0].id,startIn:e[0].startIn,types:t,multiple:e[0].multiple||!1,excludeAcceptAllOption:e[0].excludeAcceptAllOption||!1}),r=await Promise.all(n.map(ze));return e[0].multiple?r:r[0]},Ye={__proto__:null,default:Ve};function z(e){function t(n){if(Object(n)!==n)return Promise.reject(new TypeError(n+" is not an object."));var r=n.done;return Promise.resolve(n.value).then(function(s){return{value:s,done:r}})}return z=function(n){this.s=n,this.n=n.next},z.prototype={s:null,n:null,next:function(){return t(this.n.apply(this.s,arguments))},return:function(n){var r=this.s.return;return r===void 0?Promise.resolve({value:n,done:!0}):t(r.apply(this.s,arguments))},throw:function(n){var r=this.s.return;return r===void 0?Promise.reject(n):t(r.apply(this.s,arguments))}},new z(e)}const pe=async(e,t,n=e.name,r)=>{const s=[],a=[];var c,i=!1,u=!1;try{for(var l,d=function(o){var f,g,m,p=2;for(typeof Symbol<"u"&&(g=Symbol.asyncIterator,m=Symbol.iterator);p--;){if(g&&(f=o[g])!=null)return f.call(o);if(m&&(f=o[m])!=null)return new z(f.call(o));g="@@asyncIterator",m="@@iterator"}throw new TypeError("Object is not async iterable")}(e.values());i=!(l=await d.next()).done;i=!1){const o=l.value,f=`${n}/${o.name}`;o.kind==="file"?a.push(o.getFile().then(g=>(g.directoryHandle=e,g.handle=o,Object.defineProperty(g,"webkitRelativePath",{configurable:!0,enumerable:!0,get:()=>f})))):o.kind!=="directory"||!t||r&&r(o)||s.push(pe(o,t,f,r))}}catch(o){u=!0,c=o}finally{try{i&&d.return!=null&&await d.return()}finally{if(u)throw c}}return[...(await Promise.all(s)).flat(),...await Promise.all(a)]};var Ze=async(e={})=>{e.recursive=e.recursive||!1,e.mode=e.mode||"read";const t=await window.showDirectoryPicker({id:e.id,startIn:e.startIn,mode:e.mode});return(await(await t.values()).next()).done?[t]:pe(t,e.recursive,void 0,e.skipDirectory)},Qe={__proto__:null,default:Ze},Je=async(e,t=[{}],n=null,r=!1,s=null)=>{Array.isArray(t)||(t=[t]),t[0].fileName=t[0].fileName||"Untitled";const a=[];let c=null;if(e instanceof Blob&&e.type?c=e.type:e.headers&&e.headers.get("content-type")&&(c=e.headers.get("content-type")),t.forEach((l,d)=>{a[d]={description:l.description||"Files",accept:{}},l.mimeTypes?(d===0&&c&&l.mimeTypes.push(c),l.mimeTypes.map(o=>{a[d].accept[o]=l.extensions||[]})):c?a[d].accept[c]=l.extensions||[]:a[d].accept["*/*"]=l.extensions||[]}),n)try{await n.getFile()}catch(l){if(n=null,r)throw l}const i=n||await window.showSaveFilePicker({suggestedName:t[0].fileName,id:t[0].id,startIn:t[0].startIn,types:a,excludeAcceptAllOption:t[0].excludeAcceptAllOption||!1});!n&&s&&s(i);const u=await i.createWritable();return"stream"in e?(await e.stream().pipeTo(u),i):"body"in e?(await e.body.pipeTo(u),i):(await u.write(await e),await u.close(),i)},et={__proto__:null,default:Je},tt=async(e=[{}])=>(Array.isArray(e)||(e=[e]),new Promise((t,n)=>{const r=document.createElement("input");r.type="file";const s=[...e.map(u=>u.mimeTypes||[]),...e.map(u=>u.extensions||[])].join();r.multiple=e[0].multiple||!1,r.accept=s||"",r.style.display="none",document.body.append(r);const a=u=>{typeof c=="function"&&c(),t(u)},c=e[0].legacySetup&&e[0].legacySetup(a,()=>c(n),r),i=()=>{window.removeEventListener("focus",i),r.remove()};r.addEventListener("click",()=>{window.addEventListener("focus",i)}),r.addEventListener("change",()=>{window.removeEventListener("focus",i),r.remove(),a(r.multiple?Array.from(r.files):r.files[0])}),"showPicker"in HTMLInputElement.prototype?r.showPicker():r.click()})),nt={__proto__:null,default:tt},rt=async(e=[{}])=>(Array.isArray(e)||(e=[e]),e[0].recursive=e[0].recursive||!1,new Promise((t,n)=>{const r=document.createElement("input");r.type="file",r.webkitdirectory=!0;const s=c=>{typeof a=="function"&&a(),t(c)},a=e[0].legacySetup&&e[0].legacySetup(s,()=>a(n),r);r.addEventListener("change",()=>{let c=Array.from(r.files);e[0].recursive?e[0].recursive&&e[0].skipDirectory&&(c=c.filter(i=>i.webkitRelativePath.split("/").every(u=>!e[0].skipDirectory({name:u,kind:"directory"})))):c=c.filter(i=>i.webkitRelativePath.split("/").length===2),s(c)}),"showPicker"in HTMLInputElement.prototype?r.showPicker():r.click()})),st={__proto__:null,default:rt},at=async(e,t={})=>{Array.isArray(t)&&(t=t[0]);const n=document.createElement("a");let r=e;"body"in e&&(r=await async function(c,i){const u=c.getReader(),l=new ReadableStream({start:f=>async function g(){return u.read().then(({done:m,value:p})=>{if(!m)return f.enqueue(p),g();f.close()})}()}),d=new Response(l),o=await d.blob();return u.releaseLock(),new Blob([o],{type:i})}(e.body,e.headers.get("content-type"))),n.download=t.fileName||"Untitled",n.href=URL.createObjectURL(await r);const s=()=>{typeof a=="function"&&a()},a=t.legacySetup&&t.legacySetup(s,()=>a(),n);return n.addEventListener("click",()=>{setTimeout(()=>URL.revokeObjectURL(n.href),3e4),s()}),n.click(),null},ot={__proto__:null,default:at};const R=(e,t)=>({...e,enter(){t()}}),ce=22,it=e=>{const t=[R(A("Load ROMs..."),s)],n=Y(t,{visibleItems:8,onSelect:u});n.width=ce;let r=[];async function s(){try{const o=await Xe({description:"NES ROM file",extensions:[".nes"],mimeTypes:["application/octet-stream"],multiple:!0});let f=0;const g=()=>`Title Screens [${f}/${o.length}]`,m=A(g());n.update(t.concat(R(m,()=>{})));for(const p of o){try{const h=new Uint8Array(await p.arrayBuffer()),v=await e.db.rom.insert(p.name,h);o.length===1&&e.set("rom",v),await y.call("generateTitleScreen",v)}catch(h){console.error(`Failed to load file ${p.name}: ${h}`)}f+=1,m.update(g())}await c()}catch(o){console.error(o)}}function a(o){e.set("rom",o.hash)}async function c(){r=(await e.db.rom.list()).sort((o,f)=>o.name.localeCompare(f.name)),n.update(t.concat(r.map(o=>R(A(o.name,{maxLength:ce}),()=>a(o)))))}function i(){const o=n.state.activeIndex;if(o>=t.length){const f=r[o-t.length].hash;y.call("setBackground",{mode:"titleScreen",hash:f})}else y.call("setBackground",{mode:"current"})}function u(){i()}c();function l(o){return o==="Enter"?(n.state.items[n.state.activeIndex].enter(),!0):!1}function d(o){o?i():y.call("setBackground",{mode:"current"})}return{...n,onKeyDown:l,setActive:d}},we=({name:e,options:t,onChange:n,initialOption:r=t[0],text:s})=>{const a=A(e,s);let c;const i=u=>{c=u,a.update(e+": "+t[c]),n(t[u],u)};return i(t.indexOf(r)),{...a,onKeyDown(u){if(a.state.active)switch(u){case" ":return i(c===0?t.length-1:c-1),!0;case"Enter":return i((c+1)%t.length),!0}return!1}}},ct=e=>{const t={"1x":1,"2x":2,"3x":3,"4x":4,max:50};return we({name:"Zoom",options:["1x","2x","3x","4x","max"],initialOption:{1:"1x",2:"2x",3:"3x",4:"4x",50:"max"}[e.ref.scalingFactor],onChange:r=>e.set("scalingFactor",t[r])})},lt=e=>we({name:"Rendering",options:["pixelated","blurry"],initialOption:e.ref.scalingMode,onChange:t=>e.set("scalingMode",t)}),ut=()=>({...R(A("Toggle Fullscreen"),()=>y.call("toggleFullscreen")),onKeyDown(e){return e==="Enter"?(this.enter(),!0):!1}}),dt=()=>({...R(A("Soft Reset (CTRL+R)"),()=>{y.call("softReset"),y.call("toggleUI",!1)}),onKeyDown(e){return e==="Enter"?(this.enter(),!0):!1}}),ft=e=>{const t=Y([ct(e),lt(e),ut(),dt()]);return{...t,onKeyDown:s=>t.state.items[t.state.activeIndex].onKeyDown(s),setActive:s=>{}}},mt=0,gt=1,pt=e=>{const t=[R(A("Save (CTRL+S)"),()=>y.call("saveState")),R(A("Load last (CTRL+L)"),()=>y.call("loadLastSave"))],n=Y(t,{visibleItems:8,onSelect:c});n.width=19;let r=[];const s=async()=>{e.ref.rom==null?n.update(t):(r=(await e.db.save.list(e.ref.rom)).sort((l,d)=>d.timestamp-l.timestamp),n.update(t.concat(r.map(l=>{const d=new Date(l.timestamp);return R(A(`${d.toLocaleDateString()} ${d.toLocaleTimeString()}`),()=>y.call("loadSave",l.timestamp))}))))};async function a(){const l=n.state.activeIndex;switch(l){case mt:y.call("setBackground",{mode:"current"});break;case gt:if(e.ref.rom!=null){const o=await e.db.save.getLast(e.ref.rom);o!=null&&y.call("setBackground",{mode:"at",timestamp:o.timestamp})}break;default:const{timestamp:d}=r[l-t.length];y.call("setBackground",{mode:"at",timestamp:d});break}}async function c(){await a()}return s(),e.subscribe("rom",s),V.on("saved",s),{...n,onKeyDown:l=>l==="Enter"?(n.state.items[n.state.activeIndex].enter(),!0):!1,setActive:l=>{l?a():y.call("setBackground",{mode:"current"})}}},wt=[[0,0,0],[0,61,166],[0,18,176],[68,0,150],[161,0,94],[199,0,40],[186,6,0],[140,23,0],[92,47,0],[16,69,0],[5,74,0],[0,71,46],[0,65,102],[0,0,0],[5,5,5],[5,5,5],[199,199,199],[0,119,255],[33,85,255],[130,55,250],[235,47,181],[255,41,80],[255,34,0],[214,50,0],[196,98,0],[53,128,0],[5,143,0],[0,138,85],[0,153,204],[33,33,33],[9,9,9],[9,9,9],[255,255,255],[15,215,255],[105,162,255],[212,128,255],[255,69,243],[255,97,139],[255,136,51],[255,156,18],[250,188,32],[159,227,14],[43,240,53],[12,240,164],[5,251,255],[94,94,94],[13,13,13],[13,13,13],[255,255,255],[166,252,255],[179,236,255],[218,171,235],[255,168,249],[255,171,179],[255,210,176],[255,239,166],[255,247,156],[215,232,149],[166,237,175],[162,242,218],[153,255,252],[221,221,221],[17,17,17],[17,17,17]],le=256,ht=240,k=32,$=30,yt=()=>{const e=[],t=new Uint8Array(64).fill(0),n=new Uint8Array(le*ht*3).fill(0);let r=0;for(let d=0;d<k*$;d+=1)e.push(t);function s(d,o,f){d>=0&&d<k&&o>=0&&o<$&&(e[d+o*k]=f)}function a(d,o){s(d,o,t)}function c(){for(let d=0;d<k*$;d+=1)e[d]=t}function i(d,o=.2){n.set(d),r=o}function u(d,o){return Math.round(d*(1-r)+o*r)}function l(d){for(let o=0;o<$;o+=1)for(let f=0;f<k;f+=1){const g=e[f+o*k];for(let m=0;m<8;m+=1)for(let p=0;p<8;p+=1){const h=g[p+m*8],v=wt[h],x=(f*8+p+(o*8+m)*le)*3;d[x+0]=u(v[0],n[x+0]),d[x+1]=u(v[1],n[x+1]),d[x+2]=u(v[2],n[x+2])}}}return{setTile:s,clearTile:a,clear:c,render:l,setBackground:i}},_t={info:0,error:6},bt={info:48,error:48},vt=e=>{const t=yt(),n=[],r={library:it(e),saves:pt(e),controls:Ge(e),"misc.":ft(e)},s=Object.keys(r),a=qe(s.map(m=>A(m)),{initialIndex:0,onSelect:u}),c=Ne(r[s[a.state.activeIndex]]);let i=a.state.activeIndex;function u(m){r[s[i]].setActive(!1),r[s[m]].setActive(!0),i=m}V.on("uiToggled",({visible:m})=>{r[s[a.state.activeIndex]].setActive(m),n.length=0});const l={render:f,screen:t,onKeyDown:d,alert:g,visible:!0};function d(m){const p=s[a.state.activeIndex];if(m==="Escape"||m==="Tab")y.call("toggleUI");else if(l.visible){if(r[p].onKeyDown(m))return!0;switch(m){case"ArrowLeft":if(l.visible)return a.prev(),c.update(r[s[a.state.activeIndex]]),!0;break;case"ArrowRight":if(l.visible)return a.next(),c.update(r[s[a.state.activeIndex]]),!0;break;case"ArrowDown":if(l.visible)return r[p].next(),!0;break;case"ArrowUp":if(l.visible)return r[p].prev(),!0;break}}return!1}function o(){if(n.length>0){const m=n[0];if(m.frames>0){const p=bt[m.type],h=_t[m.type],v=m.text.slice(0,P);K(P-v.length,2,v,t,p,h),m.frames-=1}else n.shift()}}function f(m){t.clear(),a.render(0,6,t),c.render(0,9,t),o(),t.render(m)}function g(m){n.push(m)}return l},D=256,M=240,he=0,St=1,ye=2,xt={[he]:1024,[St]:512,[ye]:1024},ue={pixelated:"pixelated",blurry:"auto"};async function Et(){await me(),F.initPanicHook();const e=await Be(),t=vt(e),n=he,r=xt[n],s=n===ye,a=document.querySelector("#screen"),c=Me(a);let i;const u=De(e),l=new Uint8Array(D*M*3),d=new AudioContext,o=new Uint8Array(D*M*3);function f(w){try{return w()}catch(b){throw t.alert({text:`${b}`,type:"error",frames:2.5*60}),b}}a.width=D,a.height=M,a.style.imageRendering=ue[e.ref.scalingMode];function g(){const w=window.innerWidth,b=window.innerHeight,L=Math.min(w/D,b/M,e.ref.scalingFactor);a.style.width=`${D*L}px`,a.style.height=`${M*L}px`}g(),window.addEventListener("resize",g),e.subscribe("scalingFactor",g),e.subscribe("scalingMode",()=>{a.style.imageRendering=ue[e.ref.scalingMode]}),y.register("toggleUI",async w=>{w!==void 0?t.visible=w:t.visible=!t.visible,t.visible?(o.set(l),y.call("setBackground",{mode:"current"}),await d.suspend()):i!==void 0&&await d.resume(),V.emit("uiToggled",{visible:t.visible})}),window.addEventListener("blur",()=>{t.visible||y.call("toggleUI")});const m=d.createScriptProcessor(r,0,1);m.onaudioprocess=(()=>w=>{if(!t.visible){const b=w.outputBuffer.getChannelData(0);i.fillAudioBuffer(b,s)}})(),m.connect(d.destination);const p=w=>{!t.onKeyDown(w.key)&&!t.visible?u==null||u.onKeyDown(w):w.preventDefault()},h=w=>{t.visible||u==null||u.onKeyUp(w)};window.addEventListener("keyup",h),window.addEventListener("keydown",p);function v(w){i=F.new(w,d.sampleRate),l.fill(0)}async function x(w){if(w!=null)try{const b=await e.db.rom.get(w);return v(b.data),!0}catch(b){console.error("Could not load ROM:",b),t.alert({text:`${b}`,type:"error",frames:2.5*60}),e.set("rom",null)}return!1}e.subscribe("rom",async(w,b)=>{w!=null&&(w!==b?await x(w)&&y.call("toggleUI"):y.call("toggleUI"))}),y.register("loadSave",async w=>{const b=await e.db.save.get(w);f(()=>{i.loadState(b.state),y.call("toggleUI",!1)})}),y.register("loadLastSave",async()=>{const w=await e.db.save.getLast(e.ref.rom);w!=null&&f(()=>{i.loadState(w.state),y.call("toggleUI",!1)})}),y.register("saveState",async()=>{const w=i.saveState(),b=await e.db.save.insert(e.ref.rom,w);return V.emit("saved",{timestamp:b}),w});function _e(w,b){const L=i.saveState();i.loadState(w),i.nextFrame(b),i.loadState(L)}y.register("setBackground",async w=>{switch(w.mode){case"current":{l.set(o);break}case"at":{const b=await e.db.save.get(w.timestamp);_e(b.state,l);break}case"titleScreen":{if(e.ref.rom===w.hash)l.set(o);else{const b=await e.db.titleScreen.get(w.hash);b!=null?l.set(b.data):l.fill(0)}break}}t.screen.setBackground(l,.2),c.render(l)});async function be(){e.ref.rom!=null&&(await x(e.ref.rom),i&&e.ref.lastState!=null&&(i.loadState(e.ref.lastState),i.nextFrame(o),i.loadState(e.ref.lastState),y.call("setBackground",{mode:"current"}))),te()}const C=new Uint8Array(D*M*3);y.register("generateTitleScreen",async w=>{try{const b=await e.db.rom.get(w),L=await e.db.titleScreen.get(w);if(L==null){const Se=F.new(b.data,d.sampleRate);for(let ne=0;ne<120;ne++)Se.nextFrame(C);return await e.db.titleScreen.insert(w,C),C}else return L.data}catch(b){return console.error(`Failed to generate title screen for ${w}: ${b}`),C.fill(0),C}}),y.register("toggleFullscreen",()=>{document.fullscreenElement?document.exitFullscreen():a.requestFullscreen()}),y.register("softReset",()=>{i==null||i.softReset()}),y.register("setJoypad1",w=>{i==null||i.setJoypad1(w)});function ve(){i!=null&&(e.ref.lastState=i.saveState()),e.save()}function te(){requestAnimationFrame(te),t.visible?(t.render(l),c.render(l)):i!==void 0&&(u.tick(),i.nextFrame(l),c.render(l))}await be(),window.addEventListener("beforeunload",ve)}window.addEventListener("DOMContentLoaded",Et);
