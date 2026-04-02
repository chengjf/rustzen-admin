-- ============================================================================
-- Module: Partition initialization
-- Description: Pre-create operation_logs monthly partitions.
-- ============================================================================

DO $$
DECLARE
    target_date DATE;
    i INTEGER;
BEGIN
    FOR i IN 0..120 LOOP
        target_date := (CURRENT_DATE + (i || ' month')::INTERVAL)::DATE;
        PERFORM create_log_partition(target_date);
    END LOOP;
END $$;
