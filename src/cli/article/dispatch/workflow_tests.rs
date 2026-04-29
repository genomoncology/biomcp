//! Article follow-up workflow tests.

use super::article_follow_up_workflow;

fn article_with_signal() -> crate::entities::article::Article {
    crate::entities::article::Article {
        pmid: Some("12345678".to_string()),
        pmcid: None,
        doi: None,
        title: "BRAF article".to_string(),
        authors: Vec::new(),
        journal: None,
        date: None,
        citation_count: None,
        publication_type: None,
        open_access: None,
        abstract_text: None,
        full_text_path: None,
        full_text_note: None,
        full_text_source: None,
        annotations: Some(crate::entities::article::ArticleAnnotations {
            genes: vec![crate::entities::article::AnnotationCount {
                text: "BRAF".to_string(),
                count: 1,
            }],
            diseases: Vec::new(),
            chemicals: Vec::new(),
            mutations: Vec::new(),
        }),
        semantic_scholar: None,
        pubtator_fallback: false,
    }
}

#[test]
fn article_follow_up_requires_pmid_and_annotations() {
    let workflow = article_follow_up_workflow(&article_with_signal())
        .expect("workflow sidecar should load")
        .expect("article should trigger follow-up workflow");
    assert_eq!(workflow.workflow, "article-follow-up");

    let mut no_pmid = article_with_signal();
    no_pmid.pmid = None;
    assert!(
        article_follow_up_workflow(&no_pmid)
            .expect("workflow check should not fail")
            .is_none()
    );

    let mut no_annotations = article_with_signal();
    no_annotations.annotations = Some(crate::entities::article::ArticleAnnotations {
        genes: Vec::new(),
        diseases: Vec::new(),
        chemicals: Vec::new(),
        mutations: Vec::new(),
    });
    assert!(
        article_follow_up_workflow(&no_annotations)
            .expect("workflow check should not fail")
            .is_none()
    );
}
