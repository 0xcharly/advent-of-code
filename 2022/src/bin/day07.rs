use std::cell::{Ref, RefCell};
use std::rc::Rc;

extern crate anyhow;

/// A filesystem and its root node.
struct Filesystem<'fs> {
    root: Rc<RefCell<FsNode<'fs>>>,
}

/// A filesystem node, either a file (with a size), or a directory.
#[derive(Debug, PartialEq, Clone)]
enum FsNode<'fs> {
    File {
        name: &'fs str,
        size: usize,
    },
    Directory {
        name: &'fs str,
        children: Vec<Rc<RefCell<FsNode<'fs>>>>,
    },
}

impl<'fs> FsNode<'fs> {
    /// Creates a `FsNode::File` instance wrapped into a ref-counted refcell.
    fn file(name: &'fs str, size: usize) -> Rc<RefCell<FsNode<'fs>>> {
        Rc::new(RefCell::new(FsNode::File { name, size }))
    }

    /// Creates a `FsNode::Directory` instance wrapped into a ref-counted refcell.
    fn directory(
        name: &'fs str,
        children: Vec<Rc<RefCell<FsNode<'fs>>>>,
    ) -> Rc<RefCell<FsNode<'fs>>> {
        Rc::new(RefCell::new(FsNode::Directory { name, children }))
    }

    /// Returns the sum of the size of all sub-nodes.
    fn get_total_size(&self) -> usize {
        match self {
            FsNode::File { size, .. } => *size,
            FsNode::Directory { children, .. } => {
                children.iter().map(|x| x.borrow().get_total_size()).sum()
            }
        }
    }

    /// Finds a child node by its name, and returns it. Panics if the child does not exist.
    fn get_child_by_name(&self, child_name: &str) -> Rc<RefCell<FsNode<'fs>>> {
        match self {
            FsNode::File { .. } => panic!("a file has no children"),
            FsNode::Directory { children, .. } => {
                for child in children.iter() {
                    if let FsNode::Directory { name, .. } = *child.borrow_mut() {
                        if name == child_name {
                            return child.clone();
                        }
                    }
                }
                panic!("child not found: {:?}", child_name);
            }
        }
    }

    fn push_child(&mut self, child: Rc<RefCell<FsNode<'fs>>>) {
        match self {
            FsNode::File { .. } => panic!("cannot push child to a file"),
            FsNode::Directory { children, .. } => children.push(child),
        }
    }
}

// Use type alias instead of a new type for simplicity.
type DirStack<'fs> = Vec<Rc<RefCell<FsNode<'fs>>>>;

/// Returns a reference to the top node of the stack. Panics if the stack is empty.
fn top<'a, 'fs>(stack: &'a DirStack<'fs>) -> Ref<'a, FsNode<'fs>> {
    stack.last().expect("unexpected empty stack").borrow()
}

/// Pushes `node` in the top node's children list. Panics if the stack is empty.
fn push_child_in_top_fs_node<'a, 'fs>(stack: &'a DirStack<'fs>, node: Rc<RefCell<FsNode<'fs>>>) {
    stack
        .last()
        .expect("unexpected empty stack")
        .borrow_mut()
        .push_child(node)
}

/// Parses a shell session output log and infer the `Filesystem` structure from it.
fn parse_shell_session_output<'fs>(s: &'fs str) -> Filesystem<'fs> {
    let root = FsNode::directory("/", vec![]);
    let mut dir_stack: DirStack<'fs> = vec![];

    for line in s.lines() {
        let mut iter = line.split(' ').into_iter();
        match iter.next() {
            None => (), // Skip over blank lines.
            // A shell command. Only supporting `cd <ARG>` and `ls`.
            Some("$") => match iter.next() {
                Some("ls") => continue, // Nothing to do here, the interesting part comes after.
                Some("cd") => {
                    // Navigate the directory stack: pop the current directory if the argument
                    // is `..`, or enter (ie. push on the stack) the given directory if a name.
                    match iter.next() {
                        Some("/") => {
                            // Go to the root of the filesystem, which means keeping only the
                            // first ancestor.
                            dir_stack.clear();
                            dir_stack.push(root.clone());
                        }
                        Some("..") => {
                            dir_stack
                                .pop()
                                .expect("`cd ..`: unexpected empty dir stack");
                        }
                        Some(dir_name) => {
                            // Locate the child directory in the current directory, and push it
                            // on the stack, or panic if not found.
                            let node = top(&dir_stack).get_child_by_name(dir_name);
                            dir_stack.push(node);
                        }
                        None => panic!("missing argument to `cd` command"),
                    };
                }
                _ => panic!("unexpected shell command: `{:?}`", line),
            },
            // An entry in the output of the `ls` command.
            Some(ls_output) => {
                // This line is part of the output of `ls`.
                let rhs = iter
                    .next()
                    .expect(&format!("unexpected `ls` output: `{:?}`", ls_output));
                push_child_in_top_fs_node(
                    &dir_stack,
                    if ls_output == "dir" {
                        // This is a directory declaration of the form `dir <NAME>`.
                        FsNode::directory(rhs, vec![])
                    } else {
                        // This is a file declaration of the form `<SIZE> <NAME>`.
                        let size = ls_output
                            .parse()
                            .expect(&format!("unexpected file size format: `{:?}`", ls_output));
                        FsNode::file(rhs, size)
                    },
                );
            }
        }
    }

    Filesystem { root }
}

/// An iterator yielding a flat list of `FsNode` in DFS order.
struct FsIterator<'fs> {
    dir_stack: Vec<(Rc<RefCell<FsNode<'fs>>>, usize)>,
    current_dir: Rc<RefCell<FsNode<'fs>>>,
    current_child_index: usize,
}

impl<'fs> Iterator for FsIterator<'fs> {
    type Item = Rc<RefCell<FsNode<'fs>>>;

    // NOTE: This is an imperative implementation of an otherwise recursive process.
    // TODO: Could we implement this iterator recursively?
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current_dir.borrow().clone();
        let mut children = match node {
            FsNode::File { .. } => panic!("internal error: expected dir, got file"),
            FsNode::Directory { children, .. } => children,
        };
        while self.current_child_index >= children.len() {
            match self.dir_stack.pop() {
                None => return None, // End of iteration.
                Some((parent_dir, parent_dir_child_index)) => {
                    self.current_dir = parent_dir;
                    self.current_child_index = parent_dir_child_index;
                    children = match self.current_dir.borrow().clone() {
                        FsNode::File { .. } => panic!("internal error: expected dir, got file"),
                        FsNode::Directory { children, .. } => children,
                    };
                }
            }
        }
        let child = children[self.current_child_index].clone();
        self.current_child_index += 1;
        match *child.borrow() {
            FsNode::File { .. } => (),
            FsNode::Directory { .. } => {
                self.dir_stack
                    .push((self.current_dir.clone(), self.current_child_index));
                self.current_dir = child.clone();
                self.current_child_index = 0;
            }
        }
        Some(child)
    }
}

/// Returns an `Iterator` over a `Filesystem`, yielding a flat list of `FsNode` in DFS order.
impl<'fs> IntoIterator for &'fs Filesystem<'fs> {
    type Item = Rc<RefCell<FsNode<'fs>>>;
    type IntoIter = FsIterator<'fs>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            dir_stack: vec![],
            current_dir: self.root.clone(),
            current_child_index: 0,
        }
    }
}

fn main() {
    let input = include_str!("../../puzzles/day07.prod");
    let fs = parse_shell_session_output(input);

    let sum_size_dirs_below_100_000 = fs
        .into_iter()
        .filter_map(|node| {
            let node = &*node.borrow();
            match node {
                FsNode::File { .. } => None,
                FsNode::Directory { .. } => Some(node.get_total_size()),
            }
        })
        .filter(|size| *size <= 100_000)
        .sum::<usize>();

    println!("{:?}", sum_size_dirs_below_100_000);

    let size_smallest_dir_for_update = fs
        .into_iter()
        .filter_map(|node| {
            let node = &*node.borrow();
            match node {
                FsNode::File { .. } => None,
                FsNode::Directory { .. } => Some(node.get_total_size()),
            }
        })
        .filter(|size| *size <= 70_000_000 - fs.root.borrow().get_total_size())
        .max()
        .expect("at least one value");

    println!("{:?}", size_smallest_dir_for_update);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filesystem_iterator_empty() {
        let empty_fs = Filesystem {
            root: FsNode::directory("/", vec![]),
        };

        assert_eq!(empty_fs.into_iter().next(), None);
    }

    #[test]
    fn filesystem_iterator_single_file() {
        let single_file_fs = Filesystem {
            root: FsNode::directory("/", vec![FsNode::file("a", 1)]),
        };
        let mut iter = single_file_fs.into_iter();

        assert_eq!(iter.next(), Some(FsNode::file("a", 1)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn filesystem_iterator_multiple_files() {
        let multiple_files_fs = Filesystem {
            root: FsNode::directory(
                "/",
                vec![
                    FsNode::file("a", 1),
                    FsNode::file("b", 20),
                    FsNode::file("c", 300),
                ],
            ),
        };
        let mut iter = multiple_files_fs.into_iter();

        assert_eq!(iter.next(), Some(FsNode::file("a", 1)));
        assert_eq!(iter.next(), Some(FsNode::file("b", 20)));
        assert_eq!(iter.next(), Some(FsNode::file("c", 300)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn filesystem_iterator_single_directory() {
        let single_dir_fs = Filesystem {
            root: FsNode::directory("/", vec![FsNode::directory("a", vec![])]),
        };
        let mut iter = single_dir_fs.into_iter();

        assert_eq!(iter.next(), Some(FsNode::directory("a", vec![])));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn filesystem_iterator_multiple_directories() {
        let multiple_dir_fs = Filesystem {
            root: FsNode::directory(
                "/",
                vec![
                    FsNode::directory("a", vec![FsNode::directory("b", vec![])]),
                    FsNode::directory("c", vec![]),
                ],
            ),
        };
        let mut iter = multiple_dir_fs.into_iter();

        assert_eq!(iter.next(), Some(FsNode::directory("a", vec![FsNode::directory("b", vec![])])));
        assert_eq!(iter.next(), Some(FsNode::directory("b", vec![])));
        assert_eq!(iter.next(), Some(FsNode::directory("c", vec![])));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn filesystem_iterator_dirs_and_files() {
        let multiple_dir_fs = Filesystem {
            root: FsNode::directory(
                "/",
                vec![
                    FsNode::directory(
                        "a",
                        vec![
                            FsNode::directory("e", vec![FsNode::file("i", 584)]),
                            FsNode::file("f", 29116),
                            FsNode::file("g", 2557),
                            FsNode::file("h.lst", 62596),
                        ],
                    ),
                    FsNode::file("b.txt", 14848514),
                    FsNode::file("c.dat", 8504156),
                    FsNode::directory(
                        "d",
                        vec![
                            FsNode::file("j", 4060174),
                            FsNode::file("d.log", 8033020),
                            FsNode::file("d.ext", 5626152),
                            FsNode::file("k", 7214296),
                        ],
                    ),
                ],
            ),
        };
        let mut iter = multiple_dir_fs.into_iter();

        if let Some(node) = iter.next() {
            match *node.borrow() {
                FsNode::File { .. } => panic!("expected file"),
                FsNode::Directory { name, ref children } => {
                    assert_eq!(name, "a");
                    assert_eq!(children.len(), 4);
                }
            };
        }
        if let Some(node) = iter.next() {
            match *node.borrow() {
                FsNode::File { .. } => panic!("expected file"),
                FsNode::Directory { name, ref children } => {
                    assert_eq!(name, "e");
                    assert_eq!(children.len(), 1);
                }
            };
        }
        assert_eq!(iter.next(), Some(FsNode::file("i", 584)));
        assert_eq!(iter.next(), Some(FsNode::file("f", 29116)));
        assert_eq!(iter.next(), Some(FsNode::file("g", 2557)));
        assert_eq!(iter.next(), Some(FsNode::file("h.lst", 62596)));
        assert_eq!(iter.next(), Some(FsNode::file("b.txt", 14848514)));
        assert_eq!(iter.next(), Some(FsNode::file("c.dat", 8504156)));
        if let Some(node) = iter.next() {
            match *node.borrow() {
                FsNode::File { .. } => panic!("expected file"),
                FsNode::Directory { name, ref children } => {
                    assert_eq!(name, "d");
                    assert_eq!(children.len(), 4);
                }
            };
        }
        assert_eq!(iter.next(), Some(FsNode::file("j", 4060174)));
        assert_eq!(iter.next(), Some(FsNode::file("d.log", 8033020)));
        assert_eq!(iter.next(), Some(FsNode::file("d.ext", 5626152)));
        assert_eq!(iter.next(), Some(FsNode::file("k", 7214296)));
        assert_eq!(iter.next(), None);

        assert_eq!(multiple_dir_fs.root.borrow().get_total_size(), 48381165);

        let sum_largest_dirs = multiple_dir_fs
            .into_iter()
            .filter_map(|node| {
                let node = &*node.borrow();
                match node {
                    FsNode::File { .. } => None,
                    FsNode::Directory { .. } => Some(node.get_total_size()),
                }
            })
            .filter(|size| *size <= 100_000)
            .sum::<usize>();
        assert_eq!(sum_largest_dirs, 95437)
    }
}
