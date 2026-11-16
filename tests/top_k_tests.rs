use steam_reviews::utils::top_k::TopK;

#[test]
fn keeps_the_k_largest_sorted_desc() {
    let mut top_k = TopK::new(3);
    [4, 1, 7, 3, 9, 2, 5]
        .into_iter()
        .for_each(|n| top_k.consider_include(n));
    assert_eq!(top_k.into_sorted_desc(), vec![9, 7, 5]);
}

#[test]
fn zero_capacity_never_stores() {
    let mut top_k = TopK::new(0);
    top_k.consider_include(1);
    assert!(!top_k.admits(|_| true));
    assert!(top_k.into_unsorted().is_empty());
}

#[test]
fn admits_matches_consider_include() {
    let mut top_k = TopK::new(2);
    assert!(top_k.admits(|_| false), "con lugar libre siempre admite");
    top_k.consider_include(10);
    top_k.consider_include(20);
    assert!(top_k.admits(|smallest| 15 > *smallest));
    assert!(!top_k.admits(|smallest| 5 > *smallest));
}

#[test]
fn fewer_elements_than_capacity() {
    let mut top_k = TopK::new(5);
    top_k.consider_include(2);
    top_k.consider_include(8);
    assert_eq!(top_k.into_sorted_desc(), vec![8, 2]);
}
