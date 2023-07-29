#![cfg(test)]

#[allow(unused)]
use super::*;

#[test]
fn test_meta_tag() {
    assert_eq!(
        metadata_tagname("TITLE:さいたま2000"),
        Ok((":さいたま2000", "TITLE"))
    );
    assert_eq!(
        metadata_tagname("EXAM1:something"),
        Ok((":something", "EXAM1"))
    );
}

#[test]
fn test_meta_pair() {
    // Line terminated metadata
    assert_eq!(
        metadata_pair("TITLE:さいたま2000\n"),
        Ok(("", ("TITLE", "さいたま2000")))
    );
    assert_eq!(
        metadata_pair("TITLE:POP TEAM EPIC\r\n"),
        Ok(("", ("TITLE", "POP TEAM EPIC")))
    );
    // EOF terminated metadata
    assert_eq!(
        metadata_pair("EXAM1:something"),
        Ok(("", ("EXAM1", "something")))
    );
    // Empty metadata
    assert_eq!(metadata_pair("EMPTY:"), Ok(("", ("EMPTY", ""))));
}

#[test]
fn test_end_command() {
    assert!(end_command("\n#END\n").is_ok());
    assert!(end_command("\n#END P1").is_err());
}

#[test]
fn test_start_command() {
    assert_eq!(
        start_command("\n#START P2\nsomethingsomething"),
        Ok((
            "somethingsomething",
            TrackCommand::Start {
                player: Some(Player::Player2)
            }
        ))
    );

    assert_eq!(
        start_command("\n#START P1\nsomethingsomething"),
        Ok((
            "somethingsomething",
            TrackCommand::Start {
                player: Some(Player::Player1)
            }
        ))
    );

    assert_eq!(
        start_command("#START"),
        Ok(("", TrackCommand::Start { player: None }))
    );

    assert!(&[
        start_command("#START "),
        start_command("#END"),
        start_command("#START P3")
    ]
    .iter()
    .all(Result::is_err))
}

#[test]
fn test_track_command() {
    assert_eq!(
        inner_track_command("#GOGOSTART"),
        Ok(("", TrackCommand::GogoStart))
    );
    assert!(inner_track_command("#GOGOSTART testvalue").is_err());
}

#[test]
fn test_notes() {
    use TJANoteType::*;

    assert_eq!(
        notes("10201120,\n"),
        Ok((
            ",\n",
            vec![
                Some(Don),
                None,
                Some(Kat),
                None,
                Some(Don),
                Some(Don),
                Some(Kat),
                None
            ]
        ))
    );
}

#[test]
fn test_note_track() {
    use TJANoteType::*;
    let track = "#START
1100,
1100,
2,
,
#END";

    assert_eq!(
        note_track(track),
        Ok((
            "",
            vec![
                NoteTrackEntry::Command(TrackCommand::Start { player: None }),
                NoteTrackEntry::Notes(vec![Some(Don), Some(Don), None, None]),
                NoteTrackEntry::EndMeasure,
                NoteTrackEntry::Notes(vec![Some(Don), Some(Don), None, None]),
                NoteTrackEntry::EndMeasure,
                NoteTrackEntry::Notes(vec![Some(Kat)]),
                NoteTrackEntry::EndMeasure,
                NoteTrackEntry::EndMeasure,
            ]
        ))
    )
}

#[test]
pub fn test_tja_file_item_list() {
    use NoteTrackEntry::*;
    use TJANoteType::*;
    use TJAFileItem::*;

    let track = "TITLE: POP TEAM EPIC
BPM:142

WAVE:POP TEAM EPIC.ogg


#START

#GOGOSTART

1100,
1100,
2,
,

#END
";

    assert_eq!(
        tja_file(track),
        Ok((
            "",
            vec![
                Metadata("TITLE", "POP TEAM EPIC"),
                Metadata("BPM", "142"),
                Metadata("WAVE", "POP TEAM EPIC.ogg"),
                NoteTrack(vec![
                    Command(TrackCommand::Start { player: None }),
                    Command(TrackCommand::GogoStart),
                    Notes(vec![Some(Don), Some(Don), None, None]),
                    EndMeasure,
                    Notes(vec![Some(Don), Some(Don), None, None]),
                    EndMeasure,
                    Notes(vec![Some(Kat)]),
                    EndMeasure,
                    EndMeasure,
                ])
            ]
        ))
    );

    let error = "TITLE: POP TEAM EPIC
BPM:142

WAVE:POP TEAM EPIC.ogg


#START

#GOGOSTART oops this value shouldnt exist

1100,
1100,
2,
,

#END
";
    assert!(tja_file(error).is_err());
}

#[test]
fn test_tja_file_full() {
    let ok_track = "TITLE: POP TEAM EPIC
BPM:142
WAVE:POP TEAM EPIC.ogg


BALLOON:10,20
COURSE:Easy
LEVEL:1

#START

1100,
1100,
2,
7,
,
9,

#END
";

    assert!(parse_tja_file(ok_track).is_ok());

    let no_title = format!("//{}", ok_track);
    assert_eq!(
        parse_tja_file(&no_title).unwrap_err(),
        TJAParseError::MetadataNeeded("TITLE".to_string())
    );
}

#[test]
fn test_real_tja_file_succeeds() {
    let ready_to = include_str!("./Ready to.tja");
    let no_comments = preprocess_tja_file(ready_to);

    let res = tja_file(&no_comments);

    println!("{:?}", res);
    assert!(res.is_ok());
}

#[test]
fn test_measure_command() {
    let five_four = "#START
#MEASURE 5/4
#END

";

    assert_eq!(
        tja_file(five_four),
        Ok((
            "",
            vec![TJAFileItem::NoteTrack(vec![
                NoteTrackEntry::Command(TrackCommand::Start { player: None }),
                NoteTrackEntry::Command(TrackCommand::Measure(5, 4)),
            ])]
        ))
    );
}
