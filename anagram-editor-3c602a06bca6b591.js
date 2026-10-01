import { request_exclusive } from './snippets/anagram-editor-f756d830940a9805/inline0.js';


export class IntoUnderlyingByteSource {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingByteSourceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingbytesource_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get autoAllocateChunkSize() {
        const ret = wasm.intounderlyingbytesource_autoAllocateChunkSize(this.__wbg_ptr);
        return ret >>> 0;
    }
    cancel() {
        const ptr = this.__destroy_into_raw();
        wasm.intounderlyingbytesource_cancel(ptr);
    }
    /**
     * @param {ReadableByteStreamController} controller
     * @returns {Promise<any>}
     */
    pull(controller) {
        const ret = wasm.intounderlyingbytesource_pull(this.__wbg_ptr, controller);
        return ret;
    }
    /**
     * @param {ReadableByteStreamController} controller
     */
    start(controller) {
        wasm.intounderlyingbytesource_start(this.__wbg_ptr, controller);
    }
    /**
     * @returns {ReadableStreamType}
     */
    get type() {
        const ret = wasm.intounderlyingbytesource_type(this.__wbg_ptr);
        return __wbindgen_enum_ReadableStreamType[ret];
    }
}
if (Symbol.dispose) IntoUnderlyingByteSource.prototype[Symbol.dispose] = IntoUnderlyingByteSource.prototype.free;

export class IntoUnderlyingSink {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingSinkFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsink_free(ptr, 0);
    }
    /**
     * @param {any} reason
     * @returns {Promise<any>}
     */
    abort(reason) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_abort(ptr, reason);
        return ret;
    }
    /**
     * @returns {Promise<any>}
     */
    close() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_close(ptr);
        return ret;
    }
    /**
     * @param {any} chunk
     * @returns {Promise<any>}
     */
    write(chunk) {
        const ret = wasm.intounderlyingsink_write(this.__wbg_ptr, chunk);
        return ret;
    }
}
if (Symbol.dispose) IntoUnderlyingSink.prototype[Symbol.dispose] = IntoUnderlyingSink.prototype.free;

export class IntoUnderlyingSource {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingSourceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsource_free(ptr, 0);
    }
    cancel() {
        const ptr = this.__destroy_into_raw();
        wasm.intounderlyingsource_cancel(ptr);
    }
    /**
     * @param {ReadableStreamDefaultController} controller
     * @returns {Promise<any>}
     */
    pull(controller) {
        const ret = wasm.intounderlyingsource_pull(this.__wbg_ptr, controller);
        return ret;
    }
}
if (Symbol.dispose) IntoUnderlyingSource.prototype[Symbol.dispose] = IntoUnderlyingSource.prototype.free;
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_boolean_get_c9c83ebd41b34df3: function(arg0) {
            const v = arg0;
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_a57024b9c6e4a48b: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_in_ac983077f137f2e6: function(arg0, arg1) {
            const ret = arg0 in arg1;
            return ret;
        },
        __wbg___wbindgen_is_falsy_b7464e97ddc1b7a4: function(arg0) {
            const ret = !arg0;
            return ret;
        },
        __wbg___wbindgen_is_function_5e4570eb24ffa122: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_null_7d13f41e1a2d5140: function(arg0) {
            const ret = arg0 === null;
            return ret;
        },
        __wbg___wbindgen_is_undefined_6cff064c44e0d823: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_number_get_136b9679cab35cfb: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_d154f1e671052120: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_bb96b2010945f0bc: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_be22cc64ae6946a0: function(arg0) {
            arg0._wbg_cb_unref();
        },
        __wbg_activeElement_d26d0a6ad4770cf7: function(arg0) {
            const ret = arg0.activeElement;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_addEventListener_3b8edc02c33d9f77: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_addEventListener_d6fb728fba6ad35c: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3, arg4);
        }, arguments); },
        __wbg_add_43c69c0af85151c4: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.add(getStringFromWasm0(arg1, arg2));
        }, arguments); },
        __wbg_appendChild_d5cbce3d5fa81471: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.appendChild(arg1);
            return ret;
        }, arguments); },
        __wbg_arrayBuffer_393639d72165e90f: function(arg0) {
            const ret = arg0.arrayBuffer();
            return ret;
        },
        __wbg_assignedSlot_20de50b0f8a07aae: function(arg0) {
            const ret = arg0.assignedSlot;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_body_d6eca0586d628e3c: function(arg0) {
            const ret = arg0.body;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_bottom_364a8da495eed98e: function(arg0) {
            const ret = arg0.bottom;
            return ret;
        },
        __wbg_buffer_1b8155b5bd4bc786: function(arg0) {
            const ret = arg0.buffer;
            return ret;
        },
        __wbg_buffer_78291c0e094ccf99: function(arg0) {
            const ret = arg0.buffer;
            return ret;
        },
        __wbg_button_3963e81aec2b2f60: function(arg0) {
            const ret = arg0.button;
            return ret;
        },
        __wbg_buttons_688a15d7be68a0b2: function(arg0) {
            const ret = arg0.buttons;
            return ret;
        },
        __wbg_byobRequest_f8b1c89429b77545: function(arg0) {
            const ret = arg0.byobRequest;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_byteLength_0c87007a84856522: function(arg0) {
            const ret = arg0.byteLength;
            return ret;
        },
        __wbg_byteLength_336bc7d303511ba0: function(arg0) {
            const ret = arg0.byteLength;
            return ret;
        },
        __wbg_byteOffset_2b1d5b10453ce198: function(arg0) {
            const ret = arg0.byteOffset;
            return ret;
        },
        __wbg_byteOffset_8aeda3a4e0eaeb55: function(arg0) {
            const ret = arg0.byteOffset;
            return ret;
        },
        __wbg_call_1c5886ab9c57d1c7: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.call(arg1);
            return ret;
        }, arguments); },
        __wbg_call_35dba3c747ad7521: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_cancelAnimationFrame_58acec8573d45a99: function() { return handleError(function (arg0, arg1) {
            arg0.cancelAnimationFrame(arg1);
        }, arguments); },
        __wbg_cancelBubble_3c22a4a11bfa15f9: function(arg0) {
            const ret = arg0.cancelBubble;
            return ret;
        },
        __wbg_cancel_9610ed0dbe990e64: function(arg0) {
            const ret = arg0.cancel();
            return ret;
        },
        __wbg_children_9af75d66d86a8365: function(arg0) {
            const ret = arg0.children;
            return ret;
        },
        __wbg_classList_23983f0a4979ea93: function(arg0) {
            const ret = arg0.classList;
            return ret;
        },
        __wbg_clearTimeout_113b1cde814ec762: function(arg0) {
            const ret = clearTimeout(arg0);
            return ret;
        },
        __wbg_clearTimeout_4e61cface6c91ad9: function(arg0, arg1) {
            arg0.clearTimeout(arg1);
        },
        __wbg_click_cdf5981a6746a4b8: function(arg0) {
            arg0.click();
        },
        __wbg_clientHeight_834c029be3d903a7: function(arg0) {
            const ret = arg0.clientHeight;
            return ret;
        },
        __wbg_clientLeft_43baade733a3e989: function(arg0) {
            const ret = arg0.clientLeft;
            return ret;
        },
        __wbg_clientTop_c79b694545cbd2a1: function(arg0) {
            const ret = arg0.clientTop;
            return ret;
        },
        __wbg_clientWidth_ad03e8eb6c2b0c56: function(arg0) {
            const ret = arg0.clientWidth;
            return ret;
        },
        __wbg_clientX_bf415eaddddbd22c: function(arg0) {
            const ret = arg0.clientX;
            return ret;
        },
        __wbg_clientY_406b7a2827c06e05: function(arg0) {
            const ret = arg0.clientY;
            return ret;
        },
        __wbg_cloneNode_1c667adc0c119cfa: function() { return handleError(function (arg0) {
            const ret = arg0.cloneNode();
            return ret;
        }, arguments); },
        __wbg_cloneNode_dac33f79a25a9355: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.cloneNode(arg1 !== 0);
            return ret;
        }, arguments); },
        __wbg_close_2d4d9820bc65e720: function(arg0) {
            const ret = arg0.close();
            return ret;
        },
        __wbg_close_72f69f5f2de2bc73: function() { return handleError(function (arg0) {
            arg0.close();
        }, arguments); },
        __wbg_close_97cdb44c3a7878f6: function() { return handleError(function (arg0) {
            arg0.close();
        }, arguments); },
        __wbg_close_f1ad0189df723c8e: function(arg0) {
            const ret = arg0.close();
            return ret;
        },
        __wbg_closest_7eff62c8f6e11be3: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.closest(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_compatMode_72affbdf64f4081d: function(arg0, arg1) {
            const ret = arg1.compatMode;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_composedPath_fc8c6c0a810bd2dc: function(arg0) {
            const ret = arg0.composedPath();
            return ret;
        },
        __wbg_confirm_f1128d5b70df2707: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.confirm(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_contains_3ba0161eb6906b95: function(arg0, arg1) {
            const ret = arg0.contains(arg1);
            return ret;
        },
        __wbg_content_78a37721789ada5f: function(arg0) {
            const ret = arg0.content;
            return ret;
        },
        __wbg_createComment_8bb21f73bb03ef86: function(arg0, arg1, arg2) {
            const ret = arg0.createComment(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_createElementNS_f18ede2d74f15ea1: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            const ret = arg0.createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            return ret;
        }, arguments); },
        __wbg_createElement_7f42344eee7bb810: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_createTextNode_f5ee2b1cd3e249bb: function(arg0, arg1, arg2) {
            const ret = arg0.createTextNode(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_crypto_ee03eb25b52ea298: function() { return handleError(function (arg0) {
            const ret = arg0.crypto;
            return ret;
        }, arguments); },
        __wbg_ctrlKey_8f6cb44d63052c81: function(arg0) {
            const ret = arg0.ctrlKey;
            return ret;
        },
        __wbg_ctrlKey_9490b716a4845258: function(arg0) {
            const ret = arg0.ctrlKey;
            return ret;
        },
        __wbg_currentTarget_81d519ad9e5a92ec: function(arg0) {
            const ret = arg0.currentTarget;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_dataTransfer_edccf220abf064b7: function(arg0) {
            const ret = arg0.dataTransfer;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_data_9f4d28bb2f7f81de: function(arg0) {
            const ret = arg0.data;
            return ret;
        },
        __wbg_defaultPrevented_23e93db65c94fdfa: function(arg0) {
            const ret = arg0.defaultPrevented;
            return ret;
        },
        __wbg_defaultView_7d5a4fd5654ea79a: function(arg0) {
            const ret = arg0.defaultView;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_deleteProperty_83dd9487ca70fb9c: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.deleteProperty(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_deltaY_13780a1f1e6d6f8c: function(arg0) {
            const ret = arg0.deltaY;
            return ret;
        },
        __wbg_detail_792afe1f5dd13f96: function(arg0) {
            const ret = arg0.detail;
            return ret;
        },
        __wbg_devicePixelRatio_e60a2d12bfd01f78: function(arg0) {
            const ret = arg0.devicePixelRatio;
            return ret;
        },
        __wbg_device_dfb8dd62d3d58f43: function(arg0) {
            const ret = arg0.device;
            return ret;
        },
        __wbg_disconnect_0f608ec00c13c91e: function(arg0) {
            arg0.disconnect();
        },
        __wbg_disconnect_c639f34b341ef8e8: function(arg0) {
            arg0.disconnect();
        },
        __wbg_documentElement_5db4560019067e32: function(arg0) {
            const ret = arg0.documentElement;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_document_ac38448dbfd31a57: function(arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_done_669171204c3dcae2: function(arg0) {
            const ret = arg0.done;
            return ret;
        },
        __wbg_enqueue_7d68a21eda78e72f: function() { return handleError(function (arg0, arg1) {
            arg0.enqueue(arg1);
        }, arguments); },
        __wbg_error_757e9472f8410341: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_files_56a897754f75826b: function(arg0) {
            const ret = arg0.files;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_firstElementChild_c133913f3927264c: function(arg0) {
            const ret = arg0.firstElementChild;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_focus_77d7483c7b2b9f30: function() { return handleError(function (arg0) {
            arg0.focus();
        }, arguments); },
        __wbg_focus_c7d4fe3aba923a18: function() { return handleError(function (arg0, arg1) {
            arg0.focus(arg1);
        }, arguments); },
        __wbg_frameElement_89102b09dd1a1d41: function() { return handleError(function (arg0) {
            const ret = arg0.frameElement;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_from_74f3d90e0ff11240: function(arg0) {
            const ret = Array.from(arg0);
            return ret;
        },
        __wbg_getAttribute_4c6e1df05f9ee034: function(arg0, arg1, arg2, arg3) {
            const ret = arg1.getAttribute(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getBoundingClientRect_8165472d0fc91753: function(arg0) {
            const ret = arg0.getBoundingClientRect();
            return ret;
        },
        __wbg_getClientRects_e0d6090b601cbda1: function(arg0) {
            const ret = arg0.getClientRects();
            return ret;
        },
        __wbg_getComputedStyle_d0f299f3151ac25e: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.getComputedStyle(arg1);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getDevices_68b6efa083fd82d5: function(arg0) {
            const ret = arg0.getDevices();
            return ret;
        },
        __wbg_getInfo_db4ba379c104d031: function(arg0) {
            const ret = arg0.getInfo();
            return ret;
        },
        __wbg_getItem_eb388fb8c39edb35: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg1.getItem(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getPorts_6cbc7747d1f9e41c: function(arg0) {
            const ret = arg0.getPorts();
            return ret;
        },
        __wbg_getPropertyValue_50144438fb4fc8f4: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg1.getPropertyValue(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getPrototypeOf_28594fd8c494d87a: function(arg0) {
            const ret = Object.getPrototypeOf(arg0);
            return ret;
        },
        __wbg_getRandomValues_6a55335db8979694: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getRandomValues(getArrayU8FromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_getReader_533994892cd607f7: function(arg0) {
            const ret = arg0.getReader();
            return ret;
        },
        __wbg_getWriter_4a196fb41c0f5a69: function() { return handleError(function (arg0) {
            const ret = arg0.getWriter();
            return ret;
        }, arguments); },
        __wbg_get_36debceb6d43d7a1: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_get_836a517ee3483cda: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_get_971a0c45d172643f: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_get_c0c8f8d7da0c03dd: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_get_unchecked_e20b893aeafc3fca: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_get_usb_product_id_7f9b8a7ec7294ade: function(arg0) {
            const ret = arg0.usbProductId;
            return isLikeNone(ret) ? 0xFFFFFF : ret;
        },
        __wbg_get_usb_vendor_id_415418cadeadafd9: function(arg0) {
            const ret = arg0.usbVendorId;
            return isLikeNone(ret) ? 0xFFFFFF : ret;
        },
        __wbg_get_with_index_12b3f2c37724146f: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_has_b3a6e6d0d28295fa: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.has(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_height_949ea86de6d5d939: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_height_f9570f7786f9a01f: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_hid_b74c5f22cb6c12e1: function(arg0) {
            const ret = arg0.hid;
            return ret;
        },
        __wbg_host_f512e97ce1222138: function(arg0) {
            const ret = arg0.host;
            return ret;
        },
        __wbg_includes_a4b83ade703cb80b: function(arg0, arg1, arg2) {
            const ret = arg0.includes(arg1, arg2);
            return ret;
        },
        __wbg_innerHeight_047a04fc237d09df: function() { return handleError(function (arg0) {
            const ret = arg0.innerHeight;
            return ret;
        }, arguments); },
        __wbg_innerWidth_4a973110eeb5f463: function() { return handleError(function (arg0) {
            const ret = arg0.innerWidth;
            return ret;
        }, arguments); },
        __wbg_insertBefore_f3733e91a030079e: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.insertBefore(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_instanceof_Document_2c9a76b03a14f24f: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Document;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Element_a3960bb00f4964bc: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Element;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Error_61d8a02a0f3383a1: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Error;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlElement_6b02a3740edba922: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlInputElement_6077656bcaf1eb33: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLInputElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Node_ad9597995317f467: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Node;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_SerialPort_542cc0a931493331: function(arg0) {
            let result;
            try {
                result = arg0 instanceof SerialPort;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_ShadowRoot_55844b1b54688323: function(arg0) {
            let result;
            try {
                result = arg0 instanceof ShadowRoot;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Uint8Array_f935dbb0aa7cdeed: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_5625ff9937037a38: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_intersectionRatio_03e4d8b18c1b3a9c: function(arg0) {
            const ret = arg0.intersectionRatio;
            return ret;
        },
        __wbg_isConnected_6531166e09dcdfe3: function(arg0) {
            const ret = arg0.isConnected;
            return ret;
        },
        __wbg_isContentEditable_c3c1eb06dcb28e28: function(arg0) {
            const ret = arg0.isContentEditable;
            return ret;
        },
        __wbg_is_86be747e88e872fb: function(arg0, arg1) {
            const ret = Object.is(arg0, arg1);
            return ret;
        },
        __wbg_item_3731ec6bd7970c72: function(arg0, arg1) {
            const ret = arg0.item(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_item_e5f9a04853abaf70: function(arg0, arg1) {
            const ret = arg0.item(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_key_d1b2fd5ee42567c0: function(arg0, arg1) {
            const ret = arg1.key;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_left_5acbc01043b87b48: function(arg0) {
            const ret = arg0.left;
            return ret;
        },
        __wbg_length_19ed3d1851f7cc4f: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_23fe1ce249902407: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_36bd29c6848c2144: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_ecfa2c63d3d0d82c: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_f1c3dcf9a5b12883: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_localStorage_19bddab1e4cb2413: function() { return handleError(function (arg0) {
            const ret = arg0.localStorage;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_matchMedia_0e2963d34f3ddd40: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.matchMedia(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_matches_41ec2b6eaf7405b6: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.matches(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_matches_72427e51457a4411: function(arg0) {
            const ret = arg0.matches;
            return ret;
        },
        __wbg_message_c141d5e68716b595: function(arg0) {
            const ret = arg0.message;
            return ret;
        },
        __wbg_metaKey_917f037461143e51: function(arg0) {
            const ret = arg0.metaKey;
            return ret;
        },
        __wbg_metaKey_f282cd52fbd7cb27: function(arg0) {
            const ret = arg0.metaKey;
            return ret;
        },
        __wbg_name_41b795553ec88cd8: function(arg0, arg1) {
            const ret = arg1.name;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_name_eb5ea84efa88e49a: function(arg0, arg1) {
            const ret = arg1.name;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_navigator_6cfdd5fa246d910f: function(arg0) {
            const ret = arg0.navigator;
            return ret;
        },
        __wbg_new_0_f117d868b403dc07: function() {
            const ret = new Date();
            return ret;
        },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_new_358857d90afd5a2d: function(arg0, arg1) {
            const ret = new Error(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_new_4c27d8e40bf066b2: function() { return handleError(function (arg0) {
            const ret = new ResizeObserver(arg0);
            return ret;
        }, arguments); },
        __wbg_new_77cc4f4f472aeb81: function(arg0) {
            const ret = new Uint8Array(arg0);
            return ret;
        },
        __wbg_new_ebe3e0f6837f0879: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_from_slice_3eea173078478cfe: function(arg0, arg1) {
            const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_new_no_args_51d9cfd7bb73258c: function(arg0, arg1) {
            const ret = new Function(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_new_typed_cceaf62d8d95e9f2: function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Function_fn_wasm_bindgen_4547139818af1737___JsValue_____wasm_bindgen_4547139818af1737___sys__Undefined___js_sys_db356a5e121e4adb___Function_fn_wasm_bindgen_4547139818af1737___JsValue_____wasm_bindgen_4547139818af1737___sys__Undefined_______true__15(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return ret;
            } finally {
                state0.a = 0;
            }
        },
        __wbg_new_with_byte_offset_and_length_ff6e927f8d72f0c3: function(arg0, arg1, arg2) {
            const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
            return ret;
        },
        __wbg_new_with_options_fe6f387afef24fd6: function() { return handleError(function (arg0, arg1) {
            const ret = new IntersectionObserver(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_next_42cf16ee0dafc9e2: function() { return handleError(function (arg0) {
            const ret = arg0.next();
            return ret;
        }, arguments); },
        __wbg_nodeName_4eba2a58fe0b2527: function(arg0, arg1) {
            const ret = arg1.nodeName;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_now_8b265300afd5f2b9: function() {
            const ret = Date.now();
            return ret;
        },
        __wbg_observe_2c9948097e57bf5d: function(arg0, arg1, arg2) {
            arg0.observe(arg1, arg2);
        },
        __wbg_observe_77ac29d55bfbcc1c: function(arg0, arg1) {
            arg0.observe(arg1);
        },
        __wbg_observe_e15abac5309babc4: function(arg0, arg1) {
            arg0.observe(arg1);
        },
        __wbg_offsetHeight_3ac5973ad1baf528: function(arg0) {
            const ret = arg0.offsetHeight;
            return ret;
        },
        __wbg_offsetLeft_8b1c7d8afefdd3ae: function(arg0) {
            const ret = arg0.offsetLeft;
            return ret;
        },
        __wbg_offsetParent_2cb79198acbd010e: function(arg0) {
            const ret = arg0.offsetParent;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_offsetTop_a9aa1cddffa2f9d8: function(arg0) {
            const ret = arg0.offsetTop;
            return ret;
        },
        __wbg_offsetWidth_5d9c7950b81e03f2: function(arg0) {
            const ret = arg0.offsetWidth;
            return ret;
        },
        __wbg_open_6f0ff7e4e8f663b4: function(arg0, arg1) {
            const ret = arg0.open(arg1);
            return ret;
        },
        __wbg_open_cdb717627b809fca: function(arg0) {
            const ret = arg0.open();
            return ret;
        },
        __wbg_opened_750474b2fca02139: function(arg0) {
            const ret = arg0.opened;
            return ret;
        },
        __wbg_outputs_7eaf5d365d7d364b: function(arg0) {
            const ret = arg0.outputs;
            return ret;
        },
        __wbg_ownerDocument_82c473eb17b62fb0: function(arg0) {
            const ret = arg0.ownerDocument;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_parentNode_8634e029370ec1bb: function(arg0) {
            const ret = arg0.parentNode;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_parent_4ba794fc7861855d: function() { return handleError(function (arg0) {
            const ret = arg0.parent;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_pointerId_c1e1cd6b32d6d017: function(arg0) {
            const ret = arg0.pointerId;
            return ret;
        },
        __wbg_preventDefault_19878c58b8010668: function(arg0) {
            arg0.preventDefault();
        },
        __wbg_productId_6a490da52526fa79: function(arg0) {
            const ret = arg0.productId;
            return ret;
        },
        __wbg_prompt_8f7fa7d365306891: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            const ret = arg1.prompt(getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_prototypesetcall_de8e0d9553586985: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        },
        __wbg_querySelectorAll_02dbbd6bb849b369: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.querySelectorAll(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_queueMicrotask_ac694eae12e92dfb: function(arg0) {
            queueMicrotask(arg0);
        },
        __wbg_queueMicrotask_be5fe34a8f4cad4d: function(arg0) {
            const ret = arg0.queueMicrotask;
            return ret;
        },
        __wbg_random_b0d98802be10ff20: function() {
            const ret = Math.random();
            return ret;
        },
        __wbg_read_ae34ffedeb11f034: function(arg0) {
            const ret = arg0.read();
            return ret;
        },
        __wbg_readable_91931977a6360052: function(arg0) {
            const ret = arg0.readable;
            return ret;
        },
        __wbg_relatedTarget_0707f779c6356847: function(arg0) {
            const ret = arg0.relatedTarget;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_relatedTarget_fbec2fb6d5356246: function(arg0) {
            const ret = arg0.relatedTarget;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_releaseLock_c6c7735ba6ca3e43: function(arg0) {
            arg0.releaseLock();
        },
        __wbg_releaseLock_f38d2d1c08212a8a: function(arg0) {
            arg0.releaseLock();
        },
        __wbg_removeAttribute_bb10532a6f012605: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.removeAttribute(getStringFromWasm0(arg1, arg2));
        }, arguments); },
        __wbg_removeEventListener_aa653c6b402cc27e: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.removeEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_removeEventListener_c3c2b88f301819b6: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.removeEventListener(getStringFromWasm0(arg1, arg2), arg3, arg4);
        }, arguments); },
        __wbg_removeEventListener_f0778286eef3aecc: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.removeEventListener(getStringFromWasm0(arg1, arg2), arg3, arg4 !== 0);
        }, arguments); },
        __wbg_remove_07453fe173d20eee: function(arg0) {
            arg0.remove();
        },
        __wbg_remove_426d5806a1a02ede: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.remove(getStringFromWasm0(arg1, arg2));
        }, arguments); },
        __wbg_remove_b09405b87e5f3b07: function(arg0) {
            arg0.remove();
        },
        __wbg_reportId_0fefec2f4274c7a5: function(arg0) {
            const ret = arg0.reportId;
            return ret;
        },
        __wbg_requestAnimationFrame_bcb3ce6247e27dd4: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.requestAnimationFrame(arg1);
            return ret;
        }, arguments); },
        __wbg_requestDevice_931c8cbea67c306e: function(arg0, arg1) {
            const ret = arg0.requestDevice(arg1);
            return ret;
        },
        __wbg_requestMIDIAccess_a27d222c51d368ca: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.requestMIDIAccess(arg1);
            return ret;
        }, arguments); },
        __wbg_requestPort_0526a242b2edb384: function(arg0, arg1) {
            const ret = arg0.requestPort(arg1);
            return ret;
        },
        __wbg_request_exclusive_bde12c7236a4af93: function(arg0, arg1) {
            const ret = request_exclusive(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_resolve_020f95d838c6ef25: function(arg0) {
            const ret = Promise.resolve(arg0);
            return ret;
        },
        __wbg_respond_f88cbcebace42068: function() { return handleError(function (arg0, arg1) {
            arg0.respond(arg1 >>> 0);
        }, arguments); },
        __wbg_right_6340535ef1da182e: function(arg0) {
            const ret = arg0.right;
            return ret;
        },
        __wbg_scrollHeight_7d333b172b4e3ba3: function(arg0) {
            const ret = arg0.scrollHeight;
            return ret;
        },
        __wbg_scrollHeight_afbe84d0c28d8f93: function(arg0) {
            const ret = arg0.scrollHeight;
            return ret;
        },
        __wbg_scrollLeft_992b059445f1c670: function(arg0) {
            const ret = arg0.scrollLeft;
            return ret;
        },
        __wbg_scrollTop_42673e65ebb36b56: function(arg0) {
            const ret = arg0.scrollTop;
            return ret;
        },
        __wbg_scrollWidth_b353aef788aa0bc6: function(arg0) {
            const ret = arg0.scrollWidth;
            return ret;
        },
        __wbg_scrollX_181b5c8bcd3df278: function() { return handleError(function (arg0) {
            const ret = arg0.scrollX;
            return ret;
        }, arguments); },
        __wbg_scrollY_47193160b143bbe9: function() { return handleError(function (arg0) {
            const ret = arg0.scrollY;
            return ret;
        }, arguments); },
        __wbg_select_11a9ef1b672cbe11: function(arg0) {
            arg0.select();
        },
        __wbg_sendReport_48650cfdcf494a2c: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.sendReport(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_send_702f1a58289fce3f: function() { return handleError(function (arg0, arg1) {
            arg0.send(arg1);
        }, arguments); },
        __wbg_serial_fb637acff43c4d70: function(arg0) {
            const ret = arg0.serial;
            return ret;
        },
        __wbg_setAttribute_507f8367905a9c03: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setData_6ed09bd88068b75c: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setData(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setItem_b0bb6a578106db69: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setPointerCapture_761aa655f9aebc1a: function() { return handleError(function (arg0, arg1) {
            arg0.setPointerCapture(arg1);
        }, arguments); },
        __wbg_setProperty_684ce273e28a7037: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setTimeout_8be4960d8ad2bb76: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.setTimeout(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_setTimeout_ef24d2fc3ad97385: function() { return handleError(function (arg0, arg1) {
            const ret = setTimeout(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_set_8155bb79a948541b: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(arg0, arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_b9b5b5cb7b495037: function(arg0, arg1, arg2) {
            arg0.set(getArrayU8FromWasm0(arg1, arg2));
        },
        __wbg_set_baud_rate_3199e99ffe4e76e4: function(arg0, arg1) {
            arg0.baudRate = arg1 >>> 0;
        },
        __wbg_set_box_ba18accc4586ad2a: function(arg0, arg1) {
            arg0.box = __wbindgen_enum_ResizeObserverBoxOptions[arg1];
        },
        __wbg_set_capture_0fda5cbdb4353cff: function(arg0, arg1) {
            arg0.capture = arg1 !== 0;
        },
        __wbg_set_effectAllowed_a9d4049eff35d626: function(arg0, arg1, arg2) {
            arg0.effectAllowed = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_filters_a5a63352b82a2c30: function(arg0, arg1, arg2) {
            arg0.filters = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_filters_b8930780e37e918a: function(arg0, arg1, arg2) {
            arg0.filters = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_innerHTML_7d84b81d6f2a9fdf: function(arg0, arg1, arg2) {
            arg0.innerHTML = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_nodeValue_9b1ff418691c2d97: function(arg0, arg1, arg2) {
            arg0.nodeValue = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_once_7f65050c57557ff9: function(arg0, arg1) {
            arg0.once = arg1 !== 0;
        },
        __wbg_set_passive_acb4a6d8f5b98357: function(arg0, arg1) {
            arg0.passive = arg1 !== 0;
        },
        __wbg_set_prevent_scroll_012725f8a1602bdd: function(arg0, arg1) {
            arg0.preventScroll = arg1 !== 0;
        },
        __wbg_set_product_id_7c3d1e76b2bc0dda: function(arg0, arg1) {
            arg0.productId = arg1;
        },
        __wbg_set_returnValue_13ab4782410eea06: function(arg0, arg1, arg2) {
            arg0.returnValue = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_root_margin_c9fff25c45390112: function(arg0, arg1, arg2) {
            arg0.rootMargin = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_sysex_87f72b89d258cd51: function(arg0, arg1) {
            arg0.sysex = arg1 !== 0;
        },
        __wbg_set_threshold_4a16bf04c8a5b9c6: function(arg0, arg1) {
            arg0.threshold = arg1;
        },
        __wbg_set_usage_0303eafd2a4ad53b: function(arg0, arg1) {
            arg0.usage = arg1;
        },
        __wbg_set_usage_page_0e5662e8b71a5473: function(arg0, arg1) {
            arg0.usagePage = arg1;
        },
        __wbg_set_usb_product_id_37af238b19cae7b6: function(arg0, arg1) {
            arg0.usbProductId = arg1;
        },
        __wbg_set_usb_vendor_id_26d866b17afef2cf: function(arg0, arg1) {
            arg0.usbVendorId = arg1;
        },
        __wbg_set_value_22d56bead9380ee8: function(arg0, arg1, arg2) {
            arg0.value = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_vendor_id_2a075d010038dec5: function(arg0, arg1) {
            arg0.vendorId = arg1 >>> 0;
        },
        __wbg_shiftKey_8eca009f693152b4: function(arg0) {
            const ret = arg0.shiftKey;
            return ret;
        },
        __wbg_slice_6438d3c2b2847d98: function(arg0, arg1) {
            const ret = arg1.slice();
            const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_static_accessor_GLOBAL_THIS_466428f93b4eaa76: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_c7aea38d4de089bc: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_42d4fae05e59267a: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_e0db14a0eba6a812: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_stopPropagation_13b9aec9ab20a836: function(arg0) {
            arg0.stopPropagation();
        },
        __wbg_style_f09d6445af3dd2c6: function(arg0) {
            const ret = arg0.style;
            return ret;
        },
        __wbg_supports_14fe7f863af62f0c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = CSS.supports(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            return ret;
        }, arguments); },
        __wbg_tagName_d9b3dd11c1a47e27: function(arg0, arg1) {
            const ret = arg1.tagName;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_target_13424fe1cdc436ac: function(arg0) {
            const ret = arg0.target;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_target_2258eb7f6ec99f65: function(arg0) {
            const ret = arg0.target;
            return ret;
        },
        __wbg_then_7026b513a94278a8: function(arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        },
        __wbg_then_72819b8d4e081fb5: function(arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        },
        __wbg_toISOString_b46c64435d15bf81: function(arg0) {
            const ret = arg0.toISOString();
            return ret;
        },
        __wbg_toggle_b3c8c8f750f23b8a: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg0.toggle(getStringFromWasm0(arg1, arg2), arg3 !== 0);
            return ret;
        }, arguments); },
        __wbg_top_66d56bb4eca7d9c5: function(arg0) {
            const ret = arg0.top;
            return ret;
        },
        __wbg_type_93f0d0fe5e0ba871: function(arg0, arg1) {
            const ret = arg1.type;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_userAgent_adc3e375481683f8: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.userAgent;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_value_1e2369fab29b420e: function(arg0) {
            const ret = arg0.value;
            return ret;
        },
        __wbg_value_35f0fb42e7c3d468: function(arg0, arg1) {
            const ret = arg1.value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_values_46e79f9be48b5193: function(arg0) {
            const ret = arg0.values();
            return ret;
        },
        __wbg_vendorId_c1230722b7022496: function(arg0) {
            const ret = arg0.vendorId;
            return ret;
        },
        __wbg_view_7685fe4b2845c5b6: function(arg0) {
            const ret = arg0.view;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_visualViewport_6ac37a44608e245f: function(arg0) {
            const ret = arg0.visualViewport;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_warn_917d7f727ab78481: function(arg0) {
            console.warn(arg0);
        },
        __wbg_width_9eed45149c0366f2: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_width_c13a68522d4564f3: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_writable_74ecae6c6160283a: function(arg0) {
            const ret = arg0.writable;
            return ret;
        },
        __wbg_write_cb8e84e69d70b405: function(arg0, arg1) {
            const ret = arg0.write(arg1);
            return ret;
        },
        __wbg_x_e6bb472fdb2dfc97: function(arg0) {
            const ret = arg0.x;
            return ret;
        },
        __wbg_y_5cf9c01f3aa8e124: function(arg0) {
            const ret = arg0.y;
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 264, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true_);
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 294, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Array_web_sys_8e35672df23032a8___features__gen_HidDevice__HidDevice___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_4547139818af1737___JsError___true_);
            return ret;
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Array<any>"), NamedExternref("ResizeObserver")], shim_idx: 149, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Function_fn_wasm_bindgen_4547139818af1737___JsValue_____wasm_bindgen_4547139818af1737___sys__Undefined___js_sys_db356a5e121e4adb___Function_fn_wasm_bindgen_4547139818af1737___JsValue_____wasm_bindgen_4547139818af1737___sys__Undefined_______true_);
            return ret;
        },
        __wbindgen_cast_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Array<any>")], shim_idx: 294, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Array_web_sys_8e35672df23032a8___features__gen_HidDevice__HidDevice___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_4547139818af1737___JsError___true__3);
            return ret;
        },
        __wbindgen_cast_0000000000000005: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 264, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true__4);
            return ret;
        },
        __wbindgen_cast_0000000000000006: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("FocusEvent")], shim_idx: 264, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true__5);
            return ret;
        },
        __wbindgen_cast_0000000000000007: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("MouseEvent")], shim_idx: 264, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true__6);
            return ret;
        },
        __wbindgen_cast_0000000000000008: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("PointerEvent")], shim_idx: 264, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true__7);
            return ret;
        },
        __wbindgen_cast_0000000000000009: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("SerialPort")], shim_idx: 294, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Array_web_sys_8e35672df23032a8___features__gen_HidDevice__HidDevice___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_4547139818af1737___JsError___true__8);
            return ret;
        },
        __wbindgen_cast_000000000000000a: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("undefined")], shim_idx: 294, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Array_web_sys_8e35672df23032a8___features__gen_HidDevice__HidDevice___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_4547139818af1737___JsError___true__9);
            return ret;
        },
        __wbindgen_cast_000000000000000b: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Ref(NamedExternref("Event"))], shim_idx: 51, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures________invoke___web_sys_8e35672df23032a8___features__gen_Event__Event______true_);
            return ret;
        },
        __wbindgen_cast_000000000000000c: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Vector(NamedExternref("IntersectionObserverEntry"))], shim_idx: 899, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke___alloc_508e33bd5656020b___vec__Vec_web_sys_8e35672df23032a8___features__gen_ResizeObserverEntry__ResizeObserverEntry_______true_);
            return ret;
        },
        __wbindgen_cast_000000000000000d: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Vector(NamedExternref("ResizeObserverEntry"))], shim_idx: 899, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke___alloc_508e33bd5656020b___vec__Vec_web_sys_8e35672df23032a8___features__gen_ResizeObserverEntry__ResizeObserverEntry_______true__12);
            return ret;
        },
        __wbindgen_cast_000000000000000e: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 134, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke_______true__1_);
            return ret;
        },
        __wbindgen_cast_000000000000000f: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 902, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, wasm_bindgen_4547139818af1737___convert__closures_____invoke_______true_);
            return ret;
        },
        __wbindgen_cast_0000000000000010: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000011: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./anagram-editor_bg.js": import0,
    };
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke_______true__1_(arg0, arg1) {
    wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke_______true__1_(arg0, arg1);
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke_______true_(arg0, arg1) {
    wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke_______true_(arg0, arg1);
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true_(arg0, arg1, arg2) {
    wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true_(arg0, arg1, arg2);
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true__4(arg0, arg1, arg2) {
    wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true__4(arg0, arg1, arg2);
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true__5(arg0, arg1, arg2) {
    wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true__5(arg0, arg1, arg2);
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true__6(arg0, arg1, arg2) {
    wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true__6(arg0, arg1, arg2);
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true__7(arg0, arg1, arg2) {
    wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke___wasm_bindgen_4547139818af1737___JsValue______true__7(arg0, arg1, arg2);
}

function wasm_bindgen_4547139818af1737___convert__closures________invoke___web_sys_8e35672df23032a8___features__gen_Event__Event______true_(arg0, arg1, arg2) {
    wasm.wasm_bindgen_4547139818af1737___convert__closures________invoke___web_sys_8e35672df23032a8___features__gen_Event__Event______true_(arg0, arg1, arg2);
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Array_web_sys_8e35672df23032a8___features__gen_HidDevice__HidDevice___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_4547139818af1737___JsError___true_(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Array_web_sys_8e35672df23032a8___features__gen_HidDevice__HidDevice___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_4547139818af1737___JsError___true_(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Array_web_sys_8e35672df23032a8___features__gen_HidDevice__HidDevice___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_4547139818af1737___JsError___true__3(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Array_web_sys_8e35672df23032a8___features__gen_HidDevice__HidDevice___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_4547139818af1737___JsError___true__3(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Array_web_sys_8e35672df23032a8___features__gen_HidDevice__HidDevice___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_4547139818af1737___JsError___true__8(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Array_web_sys_8e35672df23032a8___features__gen_HidDevice__HidDevice___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_4547139818af1737___JsError___true__8(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Array_web_sys_8e35672df23032a8___features__gen_HidDevice__HidDevice___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_4547139818af1737___JsError___true__9(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Array_web_sys_8e35672df23032a8___features__gen_HidDevice__HidDevice___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_4547139818af1737___JsError___true__9(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Function_fn_wasm_bindgen_4547139818af1737___JsValue_____wasm_bindgen_4547139818af1737___sys__Undefined___js_sys_db356a5e121e4adb___Function_fn_wasm_bindgen_4547139818af1737___JsValue_____wasm_bindgen_4547139818af1737___sys__Undefined_______true_(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Function_fn_wasm_bindgen_4547139818af1737___JsValue_____wasm_bindgen_4547139818af1737___sys__Undefined___js_sys_db356a5e121e4adb___Function_fn_wasm_bindgen_4547139818af1737___JsValue_____wasm_bindgen_4547139818af1737___sys__Undefined_______true_(arg0, arg1, arg2, arg3);
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Function_fn_wasm_bindgen_4547139818af1737___JsValue_____wasm_bindgen_4547139818af1737___sys__Undefined___js_sys_db356a5e121e4adb___Function_fn_wasm_bindgen_4547139818af1737___JsValue_____wasm_bindgen_4547139818af1737___sys__Undefined_______true__15(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke___js_sys_db356a5e121e4adb___Function_fn_wasm_bindgen_4547139818af1737___JsValue_____wasm_bindgen_4547139818af1737___sys__Undefined___js_sys_db356a5e121e4adb___Function_fn_wasm_bindgen_4547139818af1737___JsValue_____wasm_bindgen_4547139818af1737___sys__Undefined_______true__15(arg0, arg1, arg2, arg3);
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke___alloc_508e33bd5656020b___vec__Vec_web_sys_8e35672df23032a8___features__gen_ResizeObserverEntry__ResizeObserverEntry_______true_(arg0, arg1, arg2) {
    const ptr0 = passArrayJsValueToWasm0(arg2, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke___alloc_508e33bd5656020b___vec__Vec_web_sys_8e35672df23032a8___features__gen_ResizeObserverEntry__ResizeObserverEntry_______true_(arg0, arg1, ptr0, len0);
}

function wasm_bindgen_4547139818af1737___convert__closures_____invoke___alloc_508e33bd5656020b___vec__Vec_web_sys_8e35672df23032a8___features__gen_ResizeObserverEntry__ResizeObserverEntry_______true__12(arg0, arg1, arg2) {
    const ptr0 = passArrayJsValueToWasm0(arg2, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.wasm_bindgen_4547139818af1737___convert__closures_____invoke___alloc_508e33bd5656020b___vec__Vec_web_sys_8e35672df23032a8___features__gen_ResizeObserverEntry__ResizeObserverEntry_______true__12(arg0, arg1, ptr0, len0);
}


const __wbindgen_enum_ReadableStreamType = ["bytes"];


const __wbindgen_enum_ResizeObserverBoxOptions = ["border-box", "content-box", "device-pixel-content-box"];
const IntoUnderlyingByteSourceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingbytesource_free(ptr, 1));
const IntoUnderlyingSinkFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsink_free(ptr, 1));
const IntoUnderlyingSourceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsource_free(ptr, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => wasm.__wbindgen_destroy_closure(state.a, state.b));

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

function getArrayJsValueViewFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
    }
    return result;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
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

function makeClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        try {
            return f(state.a, state.b, ...args);
        } finally {
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_destroy_closure(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function makeMutClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
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
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_destroy_closure(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    for (let i = 0; i < array.length; i++) {
        const add = addToExternrefTable0(array[i]);
        getDataViewMemory0().setUint32(ptr + 4 * i, add, true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

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
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
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

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('anagram-editor_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
