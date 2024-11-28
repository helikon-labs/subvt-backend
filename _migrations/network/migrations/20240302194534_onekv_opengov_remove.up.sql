-- Add up migration script here
ALTER TABLE sub_onekv_candidate DROP COLUMN IF EXISTS conviction_vote_count;
ALTER TABLE sub_onekv_candidate DROP COLUMN IF EXISTS conviction_votes;
ALTER TABLE sub_onekv_candidate DROP COLUMN IF EXISTS score_opengov;