-- 在迁移开始时，先确保基础函数存在（如果还没创建的话）
-- 这里可以直接放入你之前的 create_log_partition 定义，或者直接写逻辑

DO $$
DECLARE
    target_date DATE;
    i INTEGER;
BEGIN
    -- 循环 120 次，创建从当前月开始的未来 120 个月分区
    FOR i IN 0..120 LOOP
        -- 计算目标月份的日期
        target_date := (CURRENT_DATE + (i || ' month')::INTERVAL)::DATE;
        
        -- 调用创建分区的逻辑（这里为了迁移文件的独立性，直接把逻辑写在里面更安全）
        DECLARE
            p_name TEXT := 'operation_logs_' || to_char(target_date, 'YYYY_MM');
            s_date DATE := date_trunc('month', target_date);
            e_date DATE := s_date + INTERVAL '1 month';
        BEGIN
            -- 1. 创建分区表
            EXECUTE format('CREATE TABLE IF NOT EXISTS %I PARTITION OF operation_logs
                            FOR VALUES FROM (%L) TO (%L)',
                           p_name, s_date, e_date);

            -- 2. 创建索引
            EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%s_user_id ON %I(user_id)',
                           p_name, p_name);
            EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%s_created_at ON %I(created_at)',
                           p_name, p_name);
                           
            RAISE NOTICE 'Migration: 已确保分区 % 存在 (% 到 %)', p_name, s_date, e_date;
        END;
    END LOOP;
END $$;