use libc::{c_char};
use std::ffi::{CString};

use super::Trie;
use super::trie_common::TrieCommon;

ffi_fn! {
    fn radix_trie_create()-> *mut Trie<CString, usize>{
        let trie = Trie::<CString, usize>::new();
        return Box::into_raw(Box::new(trie));
    }
}

ffi_fn! {
    fn radix_trie_free(trie_ptr: *mut Trie<CString, usize>){
        unsafe { Box::from_raw(trie_ptr as *mut Trie<CString, usize>); }
    }
}

ffi_fn! {
    fn radix_trie_insert(trie_ptr:*mut Trie<CString, usize>, key_ptr:*const c_char, value:usize){
        let trie = unsafe { &mut *(trie_ptr as *mut Trie<CString, usize>) };
        let key =  unsafe { CString::from_raw(key_ptr as *mut c_char) };
        trie.insert(key, value);
    }
}

ffi_fn! {
    fn radix_trie_len(trie_ptr:*const Trie<CString, usize>)->usize{
        let trie = unsafe { &*trie_ptr };
        return trie.len();
    }
}
