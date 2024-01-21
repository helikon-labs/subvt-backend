-- Add up migration script here
ALTER TABLE sub_onekv_candidate DROP COLUMN council_stake;
ALTER TABLE sub_onekv_candidate DROP COLUMN council_votes;
ALTER TABLE sub_onekv_candidate DROP COLUMN score_council_stake;
ALTER TABLE sub_onekv_candidate DROP COLUMN score_asn;
ALTER TABLE sub_onekv_candidate RENAME democracy_votes TO conviction_votes;
ALTER TABLE sub_onekv_candidate RENAME democracy_vote_count TO conviction_vote_count;
ALTER TABLE sub_onekv_candidate RENAME score_democracy TO score_opengov;