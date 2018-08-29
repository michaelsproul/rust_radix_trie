use libc::{c_char,size_t};
use std::ffi::CStr;
use std::vec;

use super::Trie;
use super::trie_common::TrieCommon;

ffi_fn! {
    fn radix_trie_create()-> *const Trie<Vec<u8>, usize>{
        let trie = Trie::<Vec<u8>, usize>::new();
        return Box::into_raw(Box::new(trie));
    }
}

ffi_fn! {
    fn radix_trie_free(trie_ptr: *const Trie<Vec<u8>, usize>){
        unsafe { Box::from_raw(trie_ptr as *mut Trie<Vec<u8>, usize>); }
    }
}

ffi_fn! {
    fn radix_trie_insert(trie_ptr:*const Trie<Vec<u8>, usize>, key_ptr:*const c_char, value:usize){
        let mut trie = unsafe { &mut *(trie_ptr as *mut Trie<Vec<u8>, usize>) };
        let key =  unsafe { CStr::from_ptr(key_ptr) }.to_bytes().to_vec();
        trie.insert(key, value);
    }
}

ffi_fn! {
    fn radix_trie_len(trie_ptr:*const Trie<Vec<u8>, usize>)->usize{
        let trie = unsafe { &*trie_ptr };
        return trie.len();
    }
}
