import { RawInterpreter } from './snippets/dioxus-interpreter-js-161bb2cc5b21a4da/inline0.js';
import { setAttributeInner } from './snippets/dioxus-interpreter-js-161bb2cc5b21a4da/src/js/common.js';
import { get_select_data } from './snippets/dioxus-web-ac47560c608d507f/inline0.js';
import { WebDioxusChannel } from './snippets/dioxus-web-ac47560c608d507f/src/js/eval.js';
import * as __wbg_star0 from './snippets/dioxus-web-ac47560c608d507f/inline1.js';

let wasm;

let WASM_VECTOR_LEN = 0;

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_4.set(idx, obj);
    return idx;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    const mem = getDataViewMemory0();
    for (let i = 0; i < array.length; i++) {
        mem.setUint32(ptr + 4 * i, addToExternrefTable0(array[i]), true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_export_4.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_7.get(state.dtor)(state.a, state.b)
});

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_7.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}
function __wbg_adapter_50(arg0, arg1) {
    wasm._ZN132__LT_dyn_u20_core__ops__function__FnMut_LT__LP__RP__GT__u2b_Output_u20__u3d__u20_R_u20_as_u20_wasm_bindgen__closure__WasmClosure_GT_8describe6invoke17h7b1170ee6b7a2836E(arg0, arg1);
}

function __wbg_adapter_53(arg0, arg1, arg2) {
    wasm.closure1269_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_56(arg0, arg1, arg2) {
    wasm.closure1275_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_59(arg0, arg1, arg2) {
    wasm.closure1378_externref_shim(arg0, arg1, arg2);
}

const __wbindgen_enum_ScrollBehavior = ["auto", "instant", "smooth"];

const __wbindgen_enum_ScrollRestoration = ["auto", "manual"];

const JSOwnerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jsowner_free(ptr >>> 0, 1));

export class JSOwner {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(JSOwner.prototype);
        obj.__wbg_ptr = ptr;
        JSOwnerFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JSOwnerFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jsowner_free(ptr, 0);
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_String_eecc4a11987127d6 = function(arg0, arg1) {
        const ret = String(arg1);
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_addEventListener_eb4c15d7b73e58c7 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_altKey_9bd4908a800c6c3e = function(arg0) {
        const ret = arg0.altKey;
        return ret;
    };
    imports.wbg.__wbg_altKey_a2e004bad377caeb = function(arg0) {
        const ret = arg0.altKey;
        return ret;
    };
    imports.wbg.__wbg_altKey_d41d40c5569bf1c9 = function(arg0) {
        const ret = arg0.altKey;
        return ret;
    };
    imports.wbg.__wbg_animationName_51c107dad7dba334 = function(arg0, arg1) {
        const ret = arg1.animationName;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_appendChild_bd44d964a284af0e = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.appendChild(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_back_471cf9a3ef43b664 = function() { return handleError(function (arg0) {
        arg0.back();
    }, arguments) };
    imports.wbg.__wbg_blockSize_bc36c32be76e0ab6 = function(arg0) {
        const ret = arg0.blockSize;
        return ret;
    };
    imports.wbg.__wbg_blur_4047d5b4604f56b4 = function() { return handleError(function (arg0) {
        arg0.blur();
    }, arguments) };
    imports.wbg.__wbg_borderBoxSize_88e99ca31911f82c = function(arg0) {
        const ret = arg0.borderBoxSize;
        return ret;
    };
    imports.wbg.__wbg_boundingClientRect_c47b3939cae44dea = function(arg0) {
        const ret = arg0.boundingClientRect;
        return ret;
    };
    imports.wbg.__wbg_bubbles_1bf8acc8ac75172a = function(arg0) {
        const ret = arg0.bubbles;
        return ret;
    };
    imports.wbg.__wbg_buffer_aa30bbb65cb44323 = function(arg0) {
        const ret = arg0.buffer;
        return ret;
    };
    imports.wbg.__wbg_button_250e0aa055013513 = function(arg0) {
        const ret = arg0.button;
        return ret;
    };
    imports.wbg.__wbg_buttons_69b0ab98f11d61f2 = function(arg0) {
        const ret = arg0.buttons;
        return ret;
    };
    imports.wbg.__wbg_call_41c7efaf6b1182f8 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.call(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_call_c45d13337ffb12ac = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.call(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_changedTouches_ec59f3d9a9c85071 = function(arg0) {
        const ret = arg0.changedTouches;
        return ret;
    };
    imports.wbg.__wbg_charCodeAt_bc7f8c3fe75ab32c = function(arg0, arg1) {
        const ret = arg0.charCodeAt(arg1 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_checked_29b3013509a71094 = function(arg0) {
        const ret = arg0.checked;
        return ret;
    };
    imports.wbg.__wbg_clientX_1b5df6266014d669 = function(arg0) {
        const ret = arg0.clientX;
        return ret;
    };
    imports.wbg.__wbg_clientX_4c1d279ae3fd17fc = function(arg0) {
        const ret = arg0.clientX;
        return ret;
    };
    imports.wbg.__wbg_clientY_0be85232b7e8e736 = function(arg0) {
        const ret = arg0.clientY;
        return ret;
    };
    imports.wbg.__wbg_clientY_1ee78d8c2cf54e22 = function(arg0) {
        const ret = arg0.clientY;
        return ret;
    };
    imports.wbg.__wbg_clipboard_4aef0baf95f419fa = function(arg0) {
        const ret = arg0.clipboard;
        return ret;
    };
    imports.wbg.__wbg_code_692d38d3a03c5058 = function(arg0, arg1) {
        const ret = arg1.code;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_contentBoxSize_c601e1f5752d15dd = function(arg0) {
        const ret = arg0.contentBoxSize;
        return ret;
    };
    imports.wbg.__wbg_createComment_f6ad92a68996b706 = function(arg0, arg1, arg2) {
        const ret = arg0.createComment(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_createElementNS_5247857addb12b7f = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = arg0.createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createElement_e4188c0047956087 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createTextNode_f880ce54d3beae5d = function(arg0, arg1, arg2) {
        const ret = arg0.createTextNode(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_ctrlKey_aa6266c18672d3ae = function(arg0) {
        const ret = arg0.ctrlKey;
        return ret;
    };
    imports.wbg.__wbg_ctrlKey_bcc18d19a5e4a97d = function(arg0) {
        const ret = arg0.ctrlKey;
        return ret;
    };
    imports.wbg.__wbg_ctrlKey_c1d4ef1bd9c6d290 = function(arg0) {
        const ret = arg0.ctrlKey;
        return ret;
    };
    imports.wbg.__wbg_dataTransfer_3d025a6126e52ef0 = function(arg0) {
        const ret = arg0.dataTransfer;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_data_2acfc0600ea4831c = function(arg0, arg1) {
        const ret = arg1.data;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_deltaMode_94381827cfac5c9f = function(arg0) {
        const ret = arg0.deltaMode;
        return ret;
    };
    imports.wbg.__wbg_deltaX_466fdb822b8bb8c0 = function(arg0) {
        const ret = arg0.deltaX;
        return ret;
    };
    imports.wbg.__wbg_deltaY_e8fb90f5089d9277 = function(arg0) {
        const ret = arg0.deltaY;
        return ret;
    };
    imports.wbg.__wbg_deltaZ_5f9fb63e5a72ab89 = function(arg0) {
        const ret = arg0.deltaZ;
        return ret;
    };
    imports.wbg.__wbg_detail_e2216751ca918dac = function(arg0) {
        const ret = arg0.detail;
        return ret;
    };
    imports.wbg.__wbg_document_2d0a0787de036309 = function(arg0) {
        const ret = arg0.document;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_done_362f78ab584a24b5 = function(arg0) {
        const ret = arg0.done;
        return ret;
    };
    imports.wbg.__wbg_elapsedTime_5c79fce974d64194 = function(arg0) {
        const ret = arg0.elapsedTime;
        return ret;
    };
    imports.wbg.__wbg_elapsedTime_8acc2fb2d1b4b9f8 = function(arg0) {
        const ret = arg0.elapsedTime;
        return ret;
    };
    imports.wbg.__wbg_entries_27a445ca6b702f8d = function(arg0) {
        const ret = Object.entries(arg0);
        return ret;
    };
    imports.wbg.__wbg_entries_3950ab663b83a0f0 = function(arg0) {
        const ret = arg0.entries();
        return ret;
    };
    imports.wbg.__wbg_error_183186cc38c7f2df = function(arg0, arg1, arg2, arg3) {
        console.error(arg0, arg1, arg2, arg3);
    };
    imports.wbg.__wbg_error_4e9ba4ac0ad780bd = function(arg0) {
        console.error(arg0);
    };
    imports.wbg.__wbg_error_7534b8e9a36f1ab4 = function(arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_error_a0f12b4a7e088737 = function(arg0, arg1) {
        console.error(arg0, arg1);
    };
    imports.wbg.__wbg_files_50fe907d9d1676f8 = function(arg0) {
        const ret = arg0.files;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_files_adba8eba29ae4c39 = function(arg0) {
        const ret = arg0.files;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_focus_cd9adc7c1a0e86c7 = function() { return handleError(function (arg0) {
        arg0.focus();
    }, arguments) };
    imports.wbg.__wbg_force_15d6b3446b74d540 = function(arg0) {
        const ret = arg0.force;
        return ret;
    };
    imports.wbg.__wbg_forward_57405727baa74057 = function() { return handleError(function (arg0) {
        arg0.forward();
    }, arguments) };
    imports.wbg.__wbg_getAttribute_e029add545f4dca1 = function(arg0, arg1, arg2, arg3) {
        const ret = arg1.getAttribute(getStringFromWasm0(arg2, arg3));
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_getBoundingClientRect_1825d7e8c47956f1 = function(arg0) {
        const ret = arg0.getBoundingClientRect();
        return ret;
    };
    imports.wbg.__wbg_getElementById_7e591348a1ddd2f1 = function(arg0, arg1, arg2) {
        const ret = arg0.getElementById(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_getNode_3019c6fd0554479b = function(arg0, arg1) {
        const ret = arg0.getNode(arg1 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_getTime_345d4ba5f51e18ea = function(arg0) {
        const ret = arg0.getTime();
        return ret;
    };
    imports.wbg.__wbg_get_01203e6a4116a116 = function(arg0, arg1) {
        const ret = arg0[arg1 >>> 0];
        return ret;
    };
    imports.wbg.__wbg_get_8bd59c5bc2fd5aca = function(arg0, arg1) {
        const ret = arg0[arg1 >>> 0];
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_get_e7114b7bf3d9d5f5 = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.get(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getselectdata_750c8ebf72af63ae = function(arg0, arg1) {
        const ret = get_select_data(arg1);
        const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_globalThis_856ff24a65e13540 = function() { return handleError(function () {
        const ret = globalThis.globalThis;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_global_fc813a897a497d26 = function() { return handleError(function () {
        const ret = global.global;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_hash_a17e2fc62d84fede = function() { return handleError(function (arg0, arg1) {
        const ret = arg1.hash;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_height_592802beaf25c6d1 = function(arg0) {
        const ret = arg0.height;
        return ret;
    };
    imports.wbg.__wbg_height_755b7c9f6deba028 = function(arg0) {
        const ret = arg0.height;
        return ret;
    };
    imports.wbg.__wbg_height_765ff4865656438f = function(arg0) {
        const ret = arg0.height;
        return ret;
    };
    imports.wbg.__wbg_history_be2bf5502c8e04ba = function() { return handleError(function (arg0) {
        const ret = arg0.history;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_href_0007e7c0fdba047e = function() { return handleError(function (arg0, arg1) {
        const ret = arg1.href;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_identifier_58e20908dbd8dfb0 = function(arg0) {
        const ret = arg0.identifier;
        return ret;
    };
    imports.wbg.__wbg_initialize_4d57f3e1cdf6f610 = function(arg0, arg1, arg2) {
        arg0.initialize(arg1, arg2);
    };
    imports.wbg.__wbg_inlineSize_0d532c1be67d9dc1 = function(arg0) {
        const ret = arg0.inlineSize;
        return ret;
    };
    imports.wbg.__wbg_instanceof_ArrayBuffer_8b96bf6c71691dc9 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof ArrayBuffer;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_DragEvent_cf11e6c036fdd000 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof DragEvent;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Element_9848bc965765cf9a = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Element;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlElement_c018c89d56e69600 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlFormElement_fbf53619e6fdddfe = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLFormElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlInputElement_e8dfdc5a3ab7a512 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLInputElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlSelectElement_cabc9d5aca229e40 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLSelectElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlTextAreaElement_896354d7696d8f11 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLTextAreaElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Node_e963053480688ff5 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Node;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Uint8Array_faa8901ba56cb8e9 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Uint8Array;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Window_56b07700cf73649e = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Window;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_intersectionRatio_30e670678bbbc40a = function(arg0) {
        const ret = arg0.intersectionRatio;
        return ret;
    };
    imports.wbg.__wbg_intersectionRect_5a7c600de6c3e3ab = function(arg0) {
        const ret = arg0.intersectionRect;
        return ret;
    };
    imports.wbg.__wbg_isArray_6836d46c89daf1b6 = function(arg0) {
        const ret = Array.isArray(arg0);
        return ret;
    };
    imports.wbg.__wbg_isComposing_5a58b86d1fd56c68 = function(arg0) {
        const ret = arg0.isComposing;
        return ret;
    };
    imports.wbg.__wbg_isIntersecting_f8617e9bc09c8d7a = function(arg0) {
        const ret = arg0.isIntersecting;
        return ret;
    };
    imports.wbg.__wbg_isPrimary_89c47a4960673ecb = function(arg0) {
        const ret = arg0.isPrimary;
        return ret;
    };
    imports.wbg.__wbg_isSafeInteger_2fb2b4f942993af4 = function(arg0) {
        const ret = Number.isSafeInteger(arg0);
        return ret;
    };
    imports.wbg.__wbg_item_ca388b70e3cce79a = function(arg0, arg1) {
        const ret = arg0.item(arg1 >>> 0);
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_iterator_773e0b022e7009f4 = function() {
        const ret = Symbol.iterator;
        return ret;
    };
    imports.wbg.__wbg_key_15a5f86c3120ac87 = function(arg0, arg1) {
        const ret = arg1.key;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_left_27346c09f80792a5 = function(arg0) {
        const ret = arg0.left;
        return ret;
    };
    imports.wbg.__wbg_length_0a11127664108286 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_length_5a32603674ba8e75 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_length_9aaa2867670f533a = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_length_a2cf7d3728aa6ae7 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_length_def44165c1f172c5 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_location_0dafe9b211f6afaa = function(arg0) {
        const ret = arg0.location;
        return ret;
    };
    imports.wbg.__wbg_location_78348c6ba78756b0 = function(arg0) {
        const ret = arg0.location;
        return ret;
    };
    imports.wbg.__wbg_log_0cc1b7768397bcfe = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_log_cb9e190acc5753fb = function(arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.log(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_mark_7438147ce31e9d4b = function(arg0, arg1) {
        performance.mark(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_measure_fb7825c11612c823 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        let deferred0_0;
        let deferred0_1;
        let deferred1_0;
        let deferred1_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            deferred1_0 = arg2;
            deferred1_1 = arg3;
            performance.measure(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }, arguments) };
    imports.wbg.__wbg_metaKey_600ddd0045e20fb2 = function(arg0) {
        const ret = arg0.metaKey;
        return ret;
    };
    imports.wbg.__wbg_metaKey_60a283eb55e4b6ad = function(arg0) {
        const ret = arg0.metaKey;
        return ret;
    };
    imports.wbg.__wbg_metaKey_a919174210409003 = function(arg0) {
        const ret = arg0.metaKey;
        return ret;
    };
    imports.wbg.__wbg_name_84ecfb0cc2e20621 = function(arg0, arg1) {
        const ret = arg1.name;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_navigator_8fd8b43c6262f7df = function(arg0) {
        const ret = arg0.navigator;
        return ret;
    };
    imports.wbg.__wbg_new0_8845828e03777b53 = function() {
        const ret = new Date();
        return ret;
    };
    imports.wbg.__wbg_new_4c16aab09d1eb450 = function() {
        const ret = new Object();
        return ret;
    };
    imports.wbg.__wbg_new_4ea8469c57561ef4 = function() { return handleError(function () {
        const ret = new FileReader();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_new_6e254ba4a466646d = function() {
        const ret = new Array();
        return ret;
    };
    imports.wbg.__wbg_new_820d2bbee2d13ba3 = function() {
        const ret = new Map();
        return ret;
    };
    imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
        const ret = new Error();
        return ret;
    };
    imports.wbg.__wbg_new_bdc34f8e8fb8b1e4 = function(arg0) {
        const ret = new WebDioxusChannel(JSOwner.__wrap(arg0));
        return ret;
    };
    imports.wbg.__wbg_new_db41cf29086ce106 = function(arg0) {
        const ret = new Uint8Array(arg0);
        return ret;
    };
    imports.wbg.__wbg_new_f8510b5dc8ad6168 = function(arg0) {
        const ret = new RawInterpreter(arg0 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_newnoargs_29f93ce2db72cd07 = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_newwithargs_6edf5e11d4a3d6ca = function(arg0, arg1, arg2, arg3) {
        const ret = new Function(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
        return ret;
    };
    imports.wbg.__wbg_next_95ee887e1f50209d = function() { return handleError(function (arg0) {
        const ret = arg0.next();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_next_b2690a2dab163f0f = function(arg0) {
        const ret = arg0.next;
        return ret;
    };
    imports.wbg.__wbg_offsetX_a2ac78d17d20319c = function(arg0) {
        const ret = arg0.offsetX;
        return ret;
    };
    imports.wbg.__wbg_offsetY_77f55e6290c96b03 = function(arg0) {
        const ret = arg0.offsetY;
        return ret;
    };
    imports.wbg.__wbg_ownerDocument_ae56f6619a50d5ee = function(arg0) {
        const ret = arg0.ownerDocument;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_pageX_755126f5e4c73ae9 = function(arg0) {
        const ret = arg0.pageX;
        return ret;
    };
    imports.wbg.__wbg_pageX_7f152fc497c63e41 = function(arg0) {
        const ret = arg0.pageX;
        return ret;
    };
    imports.wbg.__wbg_pageY_4997cbf9ebe50af4 = function(arg0) {
        const ret = arg0.pageY;
        return ret;
    };
    imports.wbg.__wbg_pageY_a57c2de56da98b3a = function(arg0) {
        const ret = arg0.pageY;
        return ret;
    };
    imports.wbg.__wbg_parentElement_95e0718896751079 = function(arg0) {
        const ret = arg0.parentElement;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_pathname_e3cfc95ed27131c4 = function() { return handleError(function (arg0, arg1) {
        const ret = arg1.pathname;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_pointerId_df8754920af2fc71 = function(arg0) {
        const ret = arg0.pointerId;
        return ret;
    };
    imports.wbg.__wbg_pointerType_3b7fbfd8c40e1d59 = function(arg0, arg1) {
        const ret = arg1.pointerType;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_pressure_b9b110fed87f5ae8 = function(arg0) {
        const ret = arg0.pressure;
        return ret;
    };
    imports.wbg.__wbg_preventDefault_77b6f1ef90c17c7c = function(arg0) {
        arg0.preventDefault();
    };
    imports.wbg.__wbg_propertyName_107f8574d5a2f727 = function(arg0, arg1) {
        const ret = arg1.propertyName;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_pseudoElement_6e2c1fb0054f9319 = function(arg0, arg1) {
        const ret = arg1.pseudoElement;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_pseudoElement_d45bd06c32d397a0 = function(arg0, arg1) {
        const ret = arg1.pseudoElement;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_pushState_11fa9d20253f35ff = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.pushState(arg1, getStringFromWasm0(arg2, arg3), arg4 === 0 ? undefined : getStringFromWasm0(arg4, arg5));
    }, arguments) };
    imports.wbg.__wbg_push_910742639069b170 = function(arg0, arg1) {
        const ret = arg0.push(arg1);
        return ret;
    };
    imports.wbg.__wbg_queueMicrotask_6808622725a52272 = function(arg0) {
        const ret = arg0.queueMicrotask;
        return ret;
    };
    imports.wbg.__wbg_queueMicrotask_ef0e86b0263a71ee = function(arg0) {
        queueMicrotask(arg0);
    };
    imports.wbg.__wbg_radiusX_ab7313ca3bc17368 = function(arg0) {
        const ret = arg0.radiusX;
        return ret;
    };
    imports.wbg.__wbg_radiusY_7d097dac9093671a = function(arg0) {
        const ret = arg0.radiusY;
        return ret;
    };
    imports.wbg.__wbg_readAsArrayBuffer_7614a4671fee12a3 = function() { return handleError(function (arg0, arg1) {
        arg0.readAsArrayBuffer(arg1);
    }, arguments) };
    imports.wbg.__wbg_readAsText_51471e48614997f3 = function() { return handleError(function (arg0, arg1) {
        arg0.readAsText(arg1);
    }, arguments) };
    imports.wbg.__wbg_repeat_73ddf14095b6d8b0 = function(arg0) {
        const ret = arg0.repeat;
        return ret;
    };
    imports.wbg.__wbg_replaceState_d6296b69c5e60c00 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.replaceState(arg1, getStringFromWasm0(arg2, arg3), arg4 === 0 ? undefined : getStringFromWasm0(arg4, arg5));
    }, arguments) };
    imports.wbg.__wbg_requestAnimationFrame_52865535ebbc0e50 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.requestAnimationFrame(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_resolve_03bf127fbf612c20 = function(arg0) {
        const ret = Promise.resolve(arg0);
        return ret;
    };
    imports.wbg.__wbg_result_3ce106d941b8612c = function() { return handleError(function (arg0) {
        const ret = arg0.result;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_rootBounds_dac46fe0df323bb1 = function(arg0) {
        const ret = arg0.rootBounds;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_rotationAngle_29525f4f996d20d9 = function(arg0) {
        const ret = arg0.rotationAngle;
        return ret;
    };
    imports.wbg.__wbg_run_039ff2fcfda5a2fd = function(arg0) {
        arg0.run();
    };
    imports.wbg.__wbg_rustRecv_af81a17681ce3a96 = function(arg0) {
        const ret = arg0.rustRecv();
        return ret;
    };
    imports.wbg.__wbg_rustSend_d6c311e771149d6c = function(arg0, arg1) {
        arg0.rustSend(arg1);
    };
    imports.wbg.__wbg_saveTemplate_1bff2747fd009e45 = function(arg0, arg1, arg2, arg3) {
        var v0 = getArrayJsValueFromWasm0(arg1, arg2).slice();
        wasm.__wbindgen_free(arg1, arg2 * 4, 4);
        arg0.saveTemplate(v0, arg3);
    };
    imports.wbg.__wbg_screenX_6670eff2872ce200 = function(arg0) {
        const ret = arg0.screenX;
        return ret;
    };
    imports.wbg.__wbg_screenX_eae7587b13cfffe0 = function(arg0) {
        const ret = arg0.screenX;
        return ret;
    };
    imports.wbg.__wbg_screenY_70323bb15b764543 = function(arg0) {
        const ret = arg0.screenY;
        return ret;
    };
    imports.wbg.__wbg_screenY_f994d5a762f5c709 = function(arg0) {
        const ret = arg0.screenY;
        return ret;
    };
    imports.wbg.__wbg_scrollHeight_f929043699b4a9af = function(arg0) {
        const ret = arg0.scrollHeight;
        return ret;
    };
    imports.wbg.__wbg_scrollIntoView_eb0c536de62c7b8f = function(arg0, arg1) {
        arg0.scrollIntoView(arg1);
    };
    imports.wbg.__wbg_scrollLeft_6582df8dec7a1f8d = function(arg0) {
        const ret = arg0.scrollLeft;
        return ret;
    };
    imports.wbg.__wbg_scrollTo_71f5c1472ab3082c = function(arg0, arg1, arg2) {
        arg0.scrollTo(arg1, arg2);
    };
    imports.wbg.__wbg_scrollTop_176664ddc7c786d2 = function(arg0) {
        const ret = arg0.scrollTop;
        return ret;
    };
    imports.wbg.__wbg_scrollWidth_8a52bcd8ca7bc12e = function(arg0) {
        const ret = arg0.scrollWidth;
        return ret;
    };
    imports.wbg.__wbg_scrollX_76490b86e16b98eb = function() { return handleError(function (arg0) {
        const ret = arg0.scrollX;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollY_7f44bd64c62690af = function() { return handleError(function (arg0) {
        const ret = arg0.scrollY;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_search_e7a4e9255da0b9ca = function() { return handleError(function (arg0, arg1) {
        const ret = arg1.search;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_self_799f153b0b6e0183 = function() { return handleError(function () {
        const ret = self.self;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setAttributeInner_042ed9409bc5d08d = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        setAttributeInner(arg0, getStringFromWasm0(arg1, arg2), arg3, arg4 === 0 ? undefined : getStringFromWasm0(arg4, arg5));
    };
    imports.wbg.__wbg_set_1f2956726252aaf4 = function(arg0, arg1, arg2) {
        const ret = arg0.set(arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_set_3807d5f0bfc24aa7 = function(arg0, arg1, arg2) {
        arg0[arg1] = arg2;
    };
    imports.wbg.__wbg_set_e1b9d9ffeee30338 = function(arg0, arg1, arg2) {
        arg0[arg1 >>> 0] = arg2;
    };
    imports.wbg.__wbg_set_e97d203fd145cdae = function(arg0, arg1, arg2) {
        arg0.set(arg1, arg2 >>> 0);
    };
    imports.wbg.__wbg_setbehavior_23c632ae2e07bd71 = function(arg0, arg1) {
        arg0.behavior = __wbindgen_enum_ScrollBehavior[arg1];
    };
    imports.wbg.__wbg_sethref_11574861c0f48e57 = function() { return handleError(function (arg0, arg1, arg2) {
        arg0.href = getStringFromWasm0(arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_setonload_74ec1208bfa7eb51 = function(arg0, arg1) {
        arg0.onload = arg1;
    };
    imports.wbg.__wbg_setscrollRestoration_7d1b62377ed22e81 = function() { return handleError(function (arg0, arg1) {
        arg0.scrollRestoration = __wbindgen_enum_ScrollRestoration[arg1];
    }, arguments) };
    imports.wbg.__wbg_shiftKey_1cfd4ab667227faf = function(arg0) {
        const ret = arg0.shiftKey;
        return ret;
    };
    imports.wbg.__wbg_shiftKey_8e8b4301ffc94f3b = function(arg0) {
        const ret = arg0.shiftKey;
        return ret;
    };
    imports.wbg.__wbg_shiftKey_abd91808221d9ec9 = function(arg0) {
        const ret = arg0.shiftKey;
        return ret;
    };
    imports.wbg.__wbg_size_3ad59474b85a5564 = function(arg0) {
        const ret = arg0.size;
        return ret;
    };
    imports.wbg.__wbg_stack_0ed75d68575b0f3c = function(arg0, arg1) {
        const ret = arg1.stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_state_e2f1f99d79ed5b2d = function() { return handleError(function (arg0) {
        const ret = arg0.state;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_stringify_8bde2d9422edb447 = function() { return handleError(function (arg0) {
        const ret = JSON.stringify(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_tangentialPressure_991203a70e899e8e = function(arg0) {
        const ret = arg0.tangentialPressure;
        return ret;
    };
    imports.wbg.__wbg_targetTouches_f498a315bbc7600a = function(arg0) {
        const ret = arg0.targetTouches;
        return ret;
    };
    imports.wbg.__wbg_target_8f11f6c47d6f15dd = function(arg0) {
        const ret = arg0.target;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_textContent_2ef23dbdd8af1dd1 = function(arg0, arg1) {
        const ret = arg1.textContent;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_then_5c9c71165832b5a1 = function(arg0, arg1, arg2) {
        const ret = arg0.then(arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_then_d88c104795b9d5aa = function(arg0, arg1) {
        const ret = arg0.then(arg1);
        return ret;
    };
    imports.wbg.__wbg_tiltX_ce3ac917b56ae83b = function(arg0) {
        const ret = arg0.tiltX;
        return ret;
    };
    imports.wbg.__wbg_tiltY_36212e3e4bc52f21 = function(arg0) {
        const ret = arg0.tiltY;
        return ret;
    };
    imports.wbg.__wbg_time_282fd4b2161abff6 = function(arg0) {
        const ret = arg0.time;
        return ret;
    };
    imports.wbg.__wbg_top_f55f10202fb0b490 = function(arg0) {
        const ret = arg0.top;
        return ret;
    };
    imports.wbg.__wbg_touches_8bc30186e2d7ecaf = function(arg0) {
        const ret = arg0.touches;
        return ret;
    };
    imports.wbg.__wbg_twist_2a2165544ec63ecf = function(arg0) {
        const ret = arg0.twist;
        return ret;
    };
    imports.wbg.__wbg_type_717dd969accb3e63 = function(arg0, arg1) {
        const ret = arg1.type;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_type_becceb06ba9191f7 = function(arg0, arg1) {
        const ret = arg1.type;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_updatememory_217ca207d383cda5 = function(arg0, arg1) {
        arg0.update_memory(arg1);
    };
    imports.wbg.__wbg_value_3f7cc78953036bcd = function(arg0, arg1) {
        const ret = arg1.value;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_60f2a065a1144020 = function(arg0, arg1) {
        const ret = arg1.value;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_87c720f6568103d1 = function(arg0) {
        const ret = arg0.value;
        return ret;
    };
    imports.wbg.__wbg_value_c0b5c7578be1de31 = function(arg0, arg1) {
        const ret = arg1.value;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_weak_cfb42702fa4177b5 = function(arg0) {
        const ret = arg0.weak();
        return ret;
    };
    imports.wbg.__wbg_width_4707194fdd3b7425 = function(arg0) {
        const ret = arg0.width;
        return ret;
    };
    imports.wbg.__wbg_width_b3425160c5b714e8 = function(arg0) {
        const ret = arg0.width;
        return ret;
    };
    imports.wbg.__wbg_width_f30fbd36c100bbf7 = function(arg0) {
        const ret = arg0.width;
        return ret;
    };
    imports.wbg.__wbg_window_cd65fa4478648b49 = function() { return handleError(function () {
        const ret = window.window;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_writeText_cb195ad0c2c2d2b8 = function(arg0, arg1, arg2) {
        const ret = arg0.writeText(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_x_e54398a4a6c47555 = function(arg0) {
        const ret = arg0.x;
        return ret;
    };
    imports.wbg.__wbg_y_35dc70974c18aff6 = function(arg0) {
        const ret = arg0.y;
        return ret;
    };
    imports.wbg.__wbindgen_bigint_from_i64 = function(arg0) {
        const ret = arg0;
        return ret;
    };
    imports.wbg.__wbindgen_bigint_from_u64 = function(arg0) {
        const ret = BigInt.asUintN(64, arg0);
        return ret;
    };
    imports.wbg.__wbindgen_bigint_get_as_i64 = function(arg0, arg1) {
        const v = arg1;
        const ret = typeof(v) === 'bigint' ? v : undefined;
        getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
    };
    imports.wbg.__wbindgen_boolean_get = function(arg0) {
        const v = arg0;
        const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
        return ret;
    };
    imports.wbg.__wbindgen_cb_drop = function(arg0) {
        const obj = arg0.original;
        if (obj.cnt-- == 1) {
            obj.a = 0;
            return true;
        }
        const ret = false;
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2433 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1270, __wbg_adapter_50);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2434 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1270, __wbg_adapter_53);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2437 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1270, __wbg_adapter_56);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2557 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1379, __wbg_adapter_59);
        return ret;
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        const ret = debugString(arg1);
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_error_new = function(arg0, arg1) {
        const ret = new Error(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbindgen_in = function(arg0, arg1) {
        const ret = arg0 in arg1;
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_export_4;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };
    imports.wbg.__wbindgen_is_bigint = function(arg0) {
        const ret = typeof(arg0) === 'bigint';
        return ret;
    };
    imports.wbg.__wbindgen_is_function = function(arg0) {
        const ret = typeof(arg0) === 'function';
        return ret;
    };
    imports.wbg.__wbindgen_is_object = function(arg0) {
        const val = arg0;
        const ret = typeof(val) === 'object' && val !== null;
        return ret;
    };
    imports.wbg.__wbindgen_is_string = function(arg0) {
        const ret = typeof(arg0) === 'string';
        return ret;
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = arg0 === undefined;
        return ret;
    };
    imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
        const ret = arg0 === arg1;
        return ret;
    };
    imports.wbg.__wbindgen_jsval_loose_eq = function(arg0, arg1) {
        const ret = arg0 == arg1;
        return ret;
    };
    imports.wbg.__wbindgen_memory = function() {
        const ret = wasm.memory;
        return ret;
    };
    imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
        const obj = arg1;
        const ret = typeof(obj) === 'number' ? obj : undefined;
        getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
    };
    imports.wbg.__wbindgen_number_new = function(arg0) {
        const ret = arg0;
        return ret;
    };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = arg1;
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports['./snippets/dioxus-web-ac47560c608d507f/inline1.js'] = __wbg_star0;

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }


    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
