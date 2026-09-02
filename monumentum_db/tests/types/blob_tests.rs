use monumentum_db::types::Blob;

#[test]
fn new_empty_blob() {
    let blob = Blob::new(Vec::new());
    assert_eq!(blob.len(), 0);
    assert!(blob.is_empty());
    assert_eq!(blob.as_slice(), &[] as &[u8]);
}

#[test]
fn new_non_empty_blob() {
    let data = vec![1, 2, 3];
    let blob = Blob::new(data.clone());
    assert_eq!(blob.len(), 3);
    assert!(!blob.is_empty());
    assert_eq!(blob.as_slice(), data.as_slice());
}

#[test]
fn as_slice_returns_correct_slice() {
    let data = vec![10, 20, 30];
    let blob = Blob::new(data.clone());
    assert_eq!(blob.as_slice(), &data[..]);
}

#[test]
fn len_returns_byte_count() {
    assert_eq!(Blob::new(vec![0; 5]).len(), 5);
    assert_eq!(Blob::new(Vec::new()).len(), 0);
}

#[test]
fn is_empty_returns_true_only_for_empty() {
    assert!(Blob::new(Vec::new()).is_empty());
    assert!(!Blob::new(vec![1]).is_empty());
}

#[test]
fn display_formats_correctly() {
    assert_eq!(format!("{}", Blob::new(vec![1, 2, 3])), "Blob(3 bytes)");
    assert_eq!(format!("{}", Blob::new(Vec::new())), "Blob(0 bytes)");
}

#[test]
fn from_vec_u8() {
    let data = vec![5, 6, 7];
    let blob = Blob::from(data.clone());
    assert_eq!(blob.as_slice(), data.as_slice());
}

#[test]
fn from_slice_u8() {
    let data = [8, 9];
    let blob = Blob::from(&data[..]);
    assert_eq!(blob.as_slice(), &data);
}

#[test]
fn as_ref_returns_inner_slice() {
    let data = vec![1, 2, 3];
    let blob = Blob::new(data.clone());
    let r: &[u8] = blob.as_ref();
    assert_eq!(r, data.as_slice());
}

#[test]
fn clone_creates_equal_blob() {
    let blob = Blob::new(vec![1, 2, 3]);
    let cloned = blob.clone();
    assert_eq!(blob, cloned);
}

#[test]
fn partial_eq_considers_content() {
    let blob1 = Blob::new(vec![1, 2]);
    let blob2 = Blob::new(vec![1, 2]);
    let blob3 = Blob::new(vec![2, 1]);
    assert_eq!(blob1, blob2);
    assert_ne!(blob1, blob3);
}
