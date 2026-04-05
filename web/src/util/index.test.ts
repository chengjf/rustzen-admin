import { describe, expect, it } from "vitest";

import { calculatePercent, convertUnit } from "./index";

describe("util helpers", () => {
    it("calculates percentages with one decimal place", () => {
        expect(calculatePercent(25, 40)).toBe(62.5);
    });

    it("returns 0 when percentage inputs are missing", () => {
        expect(calculatePercent(undefined, 40)).toBe(0);
        expect(calculatePercent(25, 0)).toBe(0);
    });

    it("converts byte values into readable units", () => {
        expect(convertUnit(1024)).toBe("1.0KB");
        expect(convertUnit(1024 * 1024)).toBe("1.0MB");
    });

    it("returns 0 when byte input is missing", () => {
        expect(convertUnit()).toBe(0);
        expect(convertUnit(0)).toBe(0);
    });
});
