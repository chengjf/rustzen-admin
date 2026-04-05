import { describe, expect, it } from "vitest";

import { MENU_OPTIONS_URL, ROLE_OPTIONS_URL, USER_STATUS_OPTIONS_URL } from "./urls";

describe("urls constants", () => {
    it("exports the expected backend option endpoints", () => {
        expect(ROLE_OPTIONS_URL).toBe("/api/system/roles/options");
        expect(USER_STATUS_OPTIONS_URL).toBe("/api/system/users/status-options");
        expect(MENU_OPTIONS_URL).toBe("/api/system/menus/options");
    });
});
