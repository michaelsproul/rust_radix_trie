// Identity macro to allow expansion of the "mutability" token tree.
macro_rules! id {
    ($e:item) => { $e }
}

// Macro to generate methods on Tries and subtries that just defer to their node counterparts.
macro_rules! generate_trie_node_methods {
    () => {
        /// Get the key stored at this node, if any.
        pub fn key(&self) -> Option<&K> {
            self.node.key()
        }

        /// Get the value stored at this node, if any.
        pub fn value(&self) -> Option<&V> {
            self.node.value()
        }

        /// Determine if the Trie contains 0 key-value pairs.
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        /// Determine if the trie is a leaf node (has no children).
        pub fn is_leaf(&self) -> bool {
            self.node.child_count == 0
        }
    }
}
