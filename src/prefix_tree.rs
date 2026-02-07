use std::collections::HashMap;

struct PrefixTreeNode {
    children: HashMap<char, PrefixTreeNode>,
    is_full_word: bool,
}

pub struct PrefixTree {
    initial_node: PrefixTreeNode,
}

impl PrefixTree {
    pub fn new() -> Self {
        return Self {
            initial_node: PrefixTreeNode {
                children: HashMap::new(),
                is_full_word: false,
            },
        };
    }
    pub fn add_word(&mut self, word: &str) {
        let mut current_node = &mut self.initial_node;
        for character in word.chars() {
            current_node = current_node
                .children
                .entry(character)
                .or_insert(PrefixTreeNode {
                    children: HashMap::new(),
                    is_full_word: false,
                });
        }
        current_node.is_full_word = true;
    }
    pub fn get_completions<'a>(&'a self, word: &'a str) -> CompletionsIter {
        let mut current_node = &self.initial_node;
        for character in word.chars() {
            if let Some(node) = current_node.children.get(&character) {
                current_node = node;
            } else {
                return CompletionsIter { stack: Vec::new() };
            }
        }

        return CompletionsIter {
            stack: vec![(current_node, word.to_owned())],
        };
    }
}

struct CompletionsIter<'a> {
    stack: Vec<(&'a PrefixTreeNode, String)>,
}

impl<'a> Iterator for CompletionsIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node_father, prefix)) = &self.stack.pop() {
            for (character, node_children) in &node_father.children {
                let mut new_prefix = prefix.clone();
                new_prefix.push(*character);
                self.stack.push((&node_children, new_prefix));
            }
            if node_father.is_full_word {
                return Some(prefix.clone());
            }
        }
        return None;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_prefix_tree() {
        let tree = PrefixTree::new();
        assert!(tree.initial_node.children.is_empty());
        assert!(!tree.initial_node.is_full_word);
    }

    #[test]
    fn test_add_word_and_get_completions() {
        let mut tree = PrefixTree::new();
        tree.add_word("apple");
        tree.add_word("apply");
        tree.add_word("apricot");
        tree.add_word("banana");

        // Test completions for "ap"
        let mut completions_ap: Vec<String> = tree.get_completions("ap").collect();
        completions_ap.sort();
        assert_eq!(completions_ap, vec!["apple", "apply", "apricot"]);

        // Test completions for "b"
        let mut completions_b: Vec<String> = tree.get_completions("b").collect();
        completions_b.sort();
        assert_eq!(completions_b, vec!["banana"]);

        // Test completions for a full word "apple"
        let mut completions_apple: Vec<String> = tree.get_completions("apple").collect();
        completions_apple.sort();
        assert_eq!(completions_apple, vec!["apple"]);

        // Test completions for a non-existent prefix
        let completions_z: Vec<String> = tree.get_completions("z").collect();
        assert!(completions_z.is_empty());

        // Test completions for an empty prefix (should return all words)
        let mut completions_empty: Vec<String> = tree.get_completions("").collect();
        completions_empty.sort();
        assert_eq!(completions_empty, vec!["apple", "apply", "apricot", "banana"]);
    }

    #[test]
    fn test_add_substring_word() {
        let mut tree = PrefixTree::new();
        tree.add_word("test");
        tree.add_word("testing");

        let mut completions: Vec<String> = tree.get_completions("test").collect();
        completions.sort();
        assert_eq!(completions, vec!["test", "testing"]);
    }

    #[test]
    fn test_no_completions() {
        let mut tree = PrefixTree::new();
        tree.add_word("hello");
        let completions: Vec<String> = tree.get_completions("world").collect();
        assert!(completions.is_empty());
    }

    #[test]
    fn test_empty_tree() {
        let tree = PrefixTree::new();
        let completions: Vec<String> = tree.get_completions("any").collect();
        assert!(completions.is_empty());
        let completions_empty_prefix: Vec<String> = tree.get_completions("").collect();
        assert!(completions_empty_prefix.is_empty());
    }
}