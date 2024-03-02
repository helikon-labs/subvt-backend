-- Add up migration script here
ALTER TABLE sub_onekv_candidate DROP COLUMN conviction_vote_count;
ALTER TABLE sub_onekv_candidate DROP COLUMN conviction_votes;
ALTER TABLE sub_onekv_candidate DROP COLUMN score_opengov;