import { describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
    useQuery: vi.fn(),
}));

vi.mock("@tanstack/react-query", () => ({
    useQuery: mocks.useQuery,
}));

import { createQueryKey, useApiQuery } from "./react-query";

describe("react-query integration helpers", () => {
    it("creates stable query keys from strings and params", () => {
        expect(createQueryKey("dashboard/stats")).toEqual(["dashboard/stats"]);
        expect(createQueryKey(["system", "users"], { page: 1 })).toEqual([
            "system",
            "users",
            { page: 1 },
        ]);
    });

    it("passes params through to useQuery", () => {
        const queryFn = vi.fn();

        useApiQuery("dashboard/stats", queryFn, { params: { page: 1 }, staleTime: 5000 });

        expect(mocks.useQuery).toHaveBeenCalledTimes(1);
        const config = mocks.useQuery.mock.calls[0][0];
        expect(config.queryKey).toEqual(["dashboard/stats", { page: 1 }]);
        expect(config.staleTime).toBe(5000);

        config.queryFn();
        expect(queryFn).toHaveBeenCalledWith({ page: 1 });
    });
});
