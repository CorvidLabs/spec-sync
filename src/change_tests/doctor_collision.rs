// ─── D5 (REQ-change-143): a late-closed member drops its collision acknowledgement ───
//
// SpecSync 6.x checks every acknowledged sequence collision against the change IDs it can
// enumerate. A member moved into the closed-late folder is no longer enumerated, so the
// acknowledgement no longer matches and audit refuses (site CHG-0032). The late close drops that
// acknowledgement and keeps it in its record; the ledger's sequence and claim never move.

fn collision_ledger() -> ChangeSequenceLedger {
    ChangeSequenceLedger {
        schema_version: 1,
        sequence: 36,
        id: "CHG-0036-give-each-collection-its-own-badge-ladder".into(),
        acknowledged_collisions: vec![
            ChangeSequenceCollision {
                sequence: 12,
                ids: vec![
                    "CHG-0012-alpha".into(),
                    "CHG-0012-beta".into(),
                ],
            },
            ChangeSequenceCollision {
                sequence: 32,
                ids: vec![
                    "CHG-0032-refresh-the-nevermore-catalog".into(),
                    "CHG-0032-ship-a-testnet-algochat-demo".into(),
                ],
            },
        ],
    }
}

#[test]
fn a_late_closed_collision_member_drops_only_its_acknowledgement() {
    let mut ledger = collision_ledger();
    let dropped = drop_collision_acknowledgement(&mut ledger, "CHG-0032-ship-a-testnet-algochat-demo")
        .expect("the member's acknowledgement");
    assert_eq!(dropped.sequence, 32);
    assert_eq!(
        dropped.ids,
        vec![
            "CHG-0032-refresh-the-nevermore-catalog".to_string(),
            "CHG-0032-ship-a-testnet-algochat-demo".to_string(),
        ]
    );
    assert_eq!(ledger.acknowledged_collisions.len(), 1);
    assert_eq!(ledger.acknowledged_collisions[0].sequence, 12);
    assert_eq!(ledger.sequence, 36, "the sequence never moves");
    assert_eq!(
        ledger.id, "CHG-0036-give-each-collection-its-own-badge-ladder",
        "the claim never moves"
    );
}

#[test]
fn a_change_in_no_collision_leaves_the_ledger_as_it_was() {
    let mut ledger = collision_ledger();
    assert!(drop_collision_acknowledgement(&mut ledger, "update-the-readme").is_none());
    assert_eq!(ledger.acknowledged_collisions, collision_ledger().acknowledged_collisions);
}
