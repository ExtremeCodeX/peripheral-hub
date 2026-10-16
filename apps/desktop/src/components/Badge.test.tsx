import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { ModeBadge, StatusBadge } from "./Badge";

describe("badges", () => {
  it("marks a connected status as ok", () => {
    const html = renderToStaticMarkup(<StatusBadge status="connected" />);
    expect(html).toContain("badge-ok");
    expect(html).toContain("Connected");
  });

  it("marks a failed status as an error", () => {
    const html = renderToStaticMarkup(<StatusBadge status="failed" />);
    expect(html).toContain("badge-error");
  });

  it("labels simulated mode distinctly from physical", () => {
    expect(renderToStaticMarkup(<ModeBadge mode="simulated" />)).toContain("Simulated");
    expect(renderToStaticMarkup(<ModeBadge mode="physical" />)).toContain("Physical");
  });
});
