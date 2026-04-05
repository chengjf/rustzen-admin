import { act, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { useLocalStore } from "./useLocalStore";

const LocalStoreHarness = ({
    storeKey,
    defaultValue = "",
}: {
    storeKey: string;
    defaultValue?: string;
}) => {
    const [value, setValue] = useLocalStore(storeKey, defaultValue);

    return (
        <div>
            <span>{value}</span>
            <button onClick={() => setValue("updated-value")}>update-local-store</button>
        </div>
    );
};

describe("useLocalStore", () => {
    it("returns the provided default value when the key is empty", () => {
        render(<LocalStoreHarness storeKey="missing-key" defaultValue="fallback" />);

        expect(screen.getByText("fallback")).toBeInTheDocument();
    });

    it("updates the keyed value", () => {
        render(<LocalStoreHarness storeKey="active-tab" />);

        act(() => {
            screen.getByRole("button", { name: "update-local-store" }).click();
        });

        expect(screen.getByText("updated-value")).toBeInTheDocument();
    });
});
