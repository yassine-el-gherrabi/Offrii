mod common;

use rest_api::traits::{NotificationOutcome, NotificationRequest, NotificationService};

fn msg(token: &str, title: &str, body: &str) -> NotificationRequest {
    NotificationRequest {
        device_token: token.to_string(),
        title: title.to_string(),
        body: body.to_string(),
    }
}

#[tokio::test]
async fn send_batch_empty_returns_empty() {
    let spy = common::SpyNotificationService::new();
    let outcomes = spy.send_batch(&[]).await;
    assert!(outcomes.is_empty());
    assert!(spy.sent.lock().unwrap().is_empty());
}

#[tokio::test]
async fn send_batch_records_all_messages() {
    let spy = common::SpyNotificationService::new();
    let messages = vec![
        msg("token_a", "Title A", "Body A"),
        msg("token_b", "Title B", "Body B"),
        msg("token_c", "Title C", "Body C"),
    ];

    let outcomes = spy.send_batch(&messages).await;

    assert_eq!(outcomes.len(), 3);
    assert!(outcomes.iter().all(|o| *o == NotificationOutcome::Sent));

    let sent = spy.sent.lock().unwrap();
    assert_eq!(sent.len(), 3);
    assert_eq!(
        sent[0],
        ("token_a".into(), "Title A".into(), "Body A".into())
    );
    assert_eq!(
        sent[1],
        ("token_b".into(), "Title B".into(), "Body B".into())
    );
    assert_eq!(
        sent[2],
        ("token_c".into(), "Title C".into(), "Body C".into())
    );
}

#[tokio::test]
async fn send_batch_returns_configured_outcomes() {
    let spy = common::SpyNotificationService::new();
    {
        let mut outcomes = spy.outcomes.lock().unwrap();
        outcomes.push(NotificationOutcome::Sent);
        outcomes.push(NotificationOutcome::InvalidToken);
        outcomes.push(NotificationOutcome::Error("boom".into()));
    }

    let messages = vec![
        msg("t1", "T", "B"),
        msg("t2", "T", "B"),
        msg("t3", "T", "B"),
    ];

    let results = spy.send_batch(&messages).await;

    assert_eq!(results[0], NotificationOutcome::Sent);
    assert_eq!(results[1], NotificationOutcome::InvalidToken);
    assert_eq!(results[2], NotificationOutcome::Error("boom".into()));
}

#[tokio::test]
async fn send_batch_defaults_to_sent_when_no_outcomes() {
    let spy = common::SpyNotificationService::new();
    let messages = vec![msg("t1", "T", "B"), msg("t2", "T", "B")];

    let results = spy.send_batch(&messages).await;

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|o| *o == NotificationOutcome::Sent));
}
