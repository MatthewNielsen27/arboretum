pub mod grammar;
pub mod trie;

#[cfg(test)]
mod tests {
    use crate::trie::grammar::*;
    use crate::trie::trie::*;

    #[test]
    fn test_grammar() {
        let g = Grammar::default();
        assert_eq!(g.seq().len(), 26);

        let g = Grammar::from(&"Aabcdefghijklmnopqrstuvwxyz", Case::Insensitive);
        assert_eq!(g.seq().len(), 26);

        let g = Grammar::from(&"Aabcdefghijklmnopqrstuvwxyz", Case::Sensitive);
        assert_eq!(g.seq().len(), 27);
    }

    #[test]
    fn test_trie() {
        let mut trie = Trie::<()>::new(Grammar::default());

        assert!(trie.find("hello").is_none());
        assert_eq!(trie.len(), 0);

        assert!(trie.insert("hello", ()).is_ok());
        assert_eq!(trie.len(), 1);

        assert!(trie.find("hello").is_some());
        assert_eq!(trie.len(), 1);

        assert!(trie.insert("hello", ()).is_err());
        assert_eq!(trie.len(), 1);

        assert!(trie.delete("hello").is_ok());
        assert_eq!(trie.len(), 0);

        assert!(trie.delete("hello").is_err());
        assert_eq!(trie.len(), 0);
    }
}
