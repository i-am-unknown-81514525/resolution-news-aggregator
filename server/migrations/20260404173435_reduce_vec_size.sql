-- Add migration script here
ALTER TABLE unify
    ALTER COLUMN embedding TYPE vector(384) USING NULL;
