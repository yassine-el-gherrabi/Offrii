-- Migration 0001: Create ENUM types
-- All custom types used across the Offrii schema

CREATE TYPE priority           AS ENUM ('Envie', 'Besoin', 'Urgent');
CREATE TYPE item_status        AS ENUM ('Active', 'Purchased', 'Postponed', 'Abandoned');
CREATE TYPE member_role        AS ENUM ('Admin', 'Member');
CREATE TYPE reservation_status AS ENUM ('Reserved', 'Gifted', 'Cancelled');
CREATE TYPE wish_status        AS ENUM ('Pending', 'Active', 'Gifted', 'Closed');
CREATE TYPE offer_status       AS ENUM ('Pending', 'Accepted', 'Refused', 'Completed');
CREATE TYPE report_target      AS ENUM ('Wish', 'Message');
CREATE TYPE report_status      AS ENUM ('Pending', 'Reviewed', 'Dismissed');
