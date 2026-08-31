use monumentum_db::types::Blob;
use std::collections::HashSet;
use std::thread;

#[test]
fn new_empty() {
    let b = Blob::new(Vec::new());
    assert_eq!(b.len(), 0);
    assert!(b.is_empty());
}

#[test]
fn new_non_empty() {
    let data = vec![1, 2, 3];
    let b = Blob::new(data.clone());
    assert_eq!(b.len(), 3);
    assert!(!b.is_empty());
    assert_eq!(b.as_slice(), &data[..]);
}

#[test]
fn as_slice_empty() {
    let b = Blob::new(Vec::new());
    assert!(b.as_slice().is_empty());
}

#[test]
fn display_empty() {
    let b = Blob::new(Vec::new());
    assert_eq!(format!("{b}"), "Blob(0 bytes)");
}

#[test]
fn display_five_bytes() {
    let b = Blob::new(vec![0; 5]);
    assert_eq!(format!("{b}"), "Blob(5 bytes)");
}

#[test]
fn from_vec() {
    let data = vec![1, 2, 3, 4];
    let b = Blob::from(data.clone());
    assert_eq!(b.as_slice(), &data[..]);
}

#[test]
fn from_slice_copies_data() {
    let mut original = [4, 5, 6];
    let b = Blob::from(&original[..]);
    original[0] = 99;
    assert_eq!(original, [99, 5, 6]);
    assert_eq!(b.as_slice(), &[4, 5, 6]);
}

#[test]
fn from_slice_empty() {
    let b = Blob::from(&[][..]);
    assert!(b.is_empty());
}

#[test]
fn as_ref_trait() {
    let data = vec![10, 20, 30];
    let b = Blob::new(data.clone());
    let slice: &[u8] = b.as_ref();
    assert_eq!(slice, &data[..]);
}

#[test]
fn partial_eq() {
    let b1 = Blob::new(vec![1, 2, 3]);
    let b2 = Blob::new(vec![1, 2, 3]);
    assert_eq!(b1, b2);
    let b3 = Blob::new(vec![1, 2, 4]);
    assert_ne!(b1, b3);
    let b4 = Blob::new(vec![]);
    let b5 = Blob::new(vec![]);
    assert_eq!(b4, b5);
    let b6 = Blob::new(vec![0]);
    let b7 = Blob::new(vec![0]);
    assert_eq!(b6, b7);
}

#[test]
fn ord() {
    let a = Blob::new(vec![1, 2]);
    let b = Blob::new(vec![1, 3]);
    assert!(a < b);
    let c = Blob::new(vec![]);
    assert!(c < a);
    let d = Blob::new(vec![1, 2, 0]);
    assert!(a < d);
    assert!(a <= b);
    assert!(b > a);
}

#[test]
fn sort_blob() {
    let mut v = [
        Blob::new(vec![3]),
        Blob::new(vec![1]),
        Blob::new(vec![2]),
        Blob::new(vec![]),
    ];
    v.sort();
    assert_eq!(v[0].as_slice(), &[]);
    assert_eq!(v[1].as_slice(), &[1]);
    assert_eq!(v[2].as_slice(), &[2]);
    assert_eq!(v[3].as_slice(), &[3]);
}

#[test]
fn hash_eq() {
    let b1 = Blob::new(vec![1, 2, 3]);
    let b2 = Blob::new(vec![1, 2, 3]);
    let mut set = HashSet::new();
    set.insert(b1);
    set.insert(b2);
    assert_eq!(set.len(), 1);
}

#[test]
fn clone_independent() {
    let b1 = Blob::new(vec![5, 6, 7]);
    let b2 = b1.clone();
    assert_eq!(b1, b2);
    let empty = Blob::new(vec![]);
    let empty_clone = empty.clone();
    assert_eq!(empty, empty_clone);
}

#[test]
fn large_blob() {
    let data = vec![7; 1_000_000];
    let b = Blob::new(data.clone());
    assert_eq!(b.len(), 1_000_000);
    assert_eq!(b.as_slice(), &data[..]);
}

#[test]
fn all_byte_values() {
    let data: Vec<u8> = (0..=255).collect();
    let b = Blob::new(data.clone());
    assert_eq!(b.as_slice(), &data[..]);
}

#[test]
fn send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Blob>();
    let b = Blob::new(vec![1, 2, 3]);
    let handle = thread::spawn(move || {
        assert_eq!(b.len(), 3);
    });
    handle.join().unwrap();
}
