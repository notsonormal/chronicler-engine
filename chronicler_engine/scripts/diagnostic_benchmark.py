"""Run diagnostic benchmarks and rank failure scenarios by debuggability."""

import argparse
import json
import subprocess
import sys
from collections import defaultdict
from datetime import datetime
from pathlib import Path

ENGINE_DIR = Path(__file__).parent.parent
REPORT_DIR = Path(__file__).parent.parent.parent / "tmp" / "diagnostics"


def run_benchmark():
    """Run the Rust diagnostic benchmark and collect results."""
    cmd = ["cargo", "test", "--test", "diagnostic_benchmark", "--", "--nocapture"]
    print(f"Running: {' '.join(cmd)}")
    print(f"In directory: {ENGINE_DIR}")
    print("-" * 60)

    result = subprocess.run(
        cmd,
        cwd=ENGINE_DIR,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )

    if result.returncode != 0:
        print("ERROR: Benchmark test suite failed to run.")
        print(result.stdout)
        print(result.stderr)
        sys.exit(1)

    # Parse BENCHMARK_RESULT lines
    results = []
    for line in result.stdout.splitlines():
        prefix = "BENCHMARK_RESULT:"
        if line.startswith(prefix):
            json_str = line[len(prefix) :]
            try:
                results.append(json.loads(json_str))
            except json.JSONDecodeError as e:
                print(f"WARNING: Could not parse benchmark result: {e}")
                print(f"  Line: {line[:200]}")

    if not results:
        print("ERROR: No benchmark results found in output.")
        print("Raw stdout:")
        print(result.stdout)
        sys.exit(1)

    return results


def compute_aggregates(results):
    """Compute aggregate statistics from benchmark results."""
    categories = defaultdict(lambda: {"count": 0, "scores": []})
    overall_scores = []

    for r in results:
        scores = r["scores"]
        avg_score = sum(scores.values()) / len(scores)
        r["average_score"] = round(avg_score, 1)
        overall_scores.append(avg_score)

        cat = r["category"]
        categories[cat]["count"] += 1
        categories[cat]["scores"].append(avg_score)

    category_summary = {}
    for cat, data in sorted(categories.items()):
        scores = data["scores"]
        category_summary[cat] = {
            "count": data["count"],
            "average": round(sum(scores) / len(scores), 1),
            "min": round(min(scores), 1),
            "max": round(max(scores), 1),
        }

    overall = {
        "scenario_count": len(results),
        "average_score": round(sum(overall_scores) / len(overall_scores), 1),
        "min_score": round(min(overall_scores), 1),
        "max_score": round(max(overall_scores), 1),
    }

    # Sort results by average score (ascending = hardest to diagnose first)
    results_sorted = sorted(results, key=lambda r: r["average_score"])

    return {
        "overall": overall,
        "by_category": category_summary,
        "scenarios": results_sorted,
    }


def generate_markdown_report(data, timestamp):
    """Generate a human-readable markdown report."""
    overall = data["overall"]
    categories = data["by_category"]
    scenarios = data["scenarios"]

    lines = [
        "# Diagnostic Signal Quality Benchmark Report",
        "",
        f"**Generated:** {timestamp}",
        f"**Scenarios tested:** {overall['scenario_count']}",
        "",
        "## Executive Summary",
        "",
        "This benchmark measures how easy it is to diagnose failures in the Chronicler Engine ",
        "by injecting known failures and scoring the quality of diagnostic signals produced.",
        "",
        "| Metric | Value |",
        "|--------|-------|",
        f"| Overall average score | {overall['average_score']:.1f} / 10 |",
        f"| Best-case scenario | {overall['max_score']:.1f} / 10 |",
        f"| Worst-case scenario | {overall['min_score']:.1f} / 10 |",
        "",
        "### Scoring Legend",
        "",
        "| Score | Interpretation |",
        "|-------|----------------|",
        "| 8-10 | Easy to diagnose — root cause is obvious from UI or debug endpoint |",
        "| 5-7 | Moderate — requires some investigation but signals are present |",
        "| 2-4 | Hard — generic errors, must read logs or source code |",
        "| 0-1 | Very hard — silent failure or completely misleading signals |",
        "",
        "## Category Breakdown",
        "",
        "| Category | Scenarios | Average | Min | Max |",
        "|----------|-----------|---------|-----|-----|",
    ]

    for cat, stats in sorted(categories.items(), key=lambda x: x[1]["average"]):
        lines.append(
            f"| {cat} | {stats['count']} | {stats['average']:.1f} | {stats['min']:.1f} | {stats['max']:.1f} |"
        )

    lines.extend(
        [
            "",
            "## Scenario Rankings (Hardest to Diagnose First)",
            "",
            "| Rank | Scenario | Category | Avg | Specificity | State | Logs | Diagnosable Without Logs? |",
            "|------|----------|----------|-----|-------------|-------|-----|---------------------------|",
        ]
    )

    for i, s in enumerate(scenarios, 1):
        scores = s["scores"]
        no_logs = "✅" if s["root_cause_discoverable_without_logs"] else "❌"
        lines.append(
            f"| {i} | `{s['scenario']}` | {s['category']} | {s['average_score']:.1f} | "
            f"{scores['error_specificity']} | {scores['state_visibility']} | {scores['log_independence']} | {no_logs} |"
        )

    lines.extend(
        [
            "",
            "## Detailed Findings",
            "",
        ]
    )

    for s in scenarios:
        s_scores = s["scores"]
        lines.extend(
            [
                f"### {s['average_score']:.1f} — `{s['scenario']}` ({s['category']})",
                "",
                f"- **Injected failure:** {s['injected_failure']}",
                f"- **User-facing error:** `{s['error_message']}`",
                f"- **Phase at failure:** {s['generation_phase']}",
                "",
                "| Dimension | Score |",
                "|-----------|-------|",
                f"| Error specificity | {s_scores['error_specificity']} / 10 |",
                f"| State visibility | {s_scores['state_visibility']} / 10 |",
                f"| Log independence | {s_scores['log_independence']} / 10 |",
                "",
                f"**Notes:** {s['notes']}",
                "",
            ]
        )

    lines.extend(
        [
            "## Recommendations",
            "",
            "Based on this baseline, the highest-impact improvements would target:",
            "",
        ]
    )

    # Top 3 worst scenarios
    worst = scenarios[:3]
    for i, s in enumerate(worst, 1):
        lines.append(
            f"{i}. **`{s['scenario']}`** ({s['category']}) — Score: {s['average_score']:.1f}/10. "
            f"{s['notes'][:200]}{'...' if len(s['notes']) > 200 else ''}"
        )
        lines.append("")

    lines.append("---")
    lines.append(f"*Report generated by diagnostic_benchmark.py at {timestamp}*")
    lines.append("")

    return "\n".join(lines)


def save_json(data, path):
    """Save raw benchmark data as JSON."""
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)
    print(f"Saved JSON: {path}")


def compare_runs(current, baseline_path):
    """Compare current run to a baseline and show deltas."""
    with open(baseline_path, encoding="utf-8") as f:
        baseline = json.load(f)

    print("\n=== Comparison to Baseline ===")
    print(f"Baseline: {baseline_path}")
    print("")

    current_overall = current["overall"]["average_score"]
    baseline_overall = baseline["overall"]["average_score"]
    delta = current_overall - baseline_overall
    direction = "improved" if delta > 0 else "regressed" if delta < 0 else "unchanged"
    print(
        f"Overall average: {current_overall:.1f} (was {baseline_overall:.1f}) — {direction} by {abs(delta):.1f}"
    )
    print("")

    baseline_scenarios = {s["scenario"]: s for s in baseline["scenarios"]}
    print("| Scenario | Before | After | Delta |")
    print("|----------|--------|-------|-------|")
    for s in current["scenarios"]:
        name = s["scenario"]
        after = s["average_score"]
        before = baseline_scenarios.get(name, {}).get("average_score", 0)
        d = after - before
        sign = "+" if d > 0 else ""
        print(f"| `{name}` | {before:.1f} | {after:.1f} | {sign}{d:.1f} |")


def main():
    parser = argparse.ArgumentParser(description="Diagnostic Signal Quality Benchmark")
    parser.add_argument("--json", action="store_true", help="Output raw JSON only")
    parser.add_argument("--compare", type=str, help="Compare to a previous baseline JSON file")
    parser.add_argument(
        "--output-dir", type=str, default=str(REPORT_DIR), help="Directory for reports"
    )
    args = parser.parse_args()

    if not ENGINE_DIR.exists():
        print(f"ERROR: Engine directory not found: {ENGINE_DIR}")
        sys.exit(1)

    results = run_benchmark()
    data = compute_aggregates(results)
    timestamp = datetime.now().isoformat()

    if args.json:
        print(json.dumps(data, indent=2))
        return

    if args.compare:
        compare_runs(data, args.compare)
        return

    # Generate and save reports
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # JSON output
    json_path = output_dir / f"diagnostic_baseline_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
    save_json(data, json_path)

    # Also save as latest.json for easy comparison
    latest_json = output_dir / "diagnostic_baseline_latest.json"
    save_json(data, latest_json)

    # Markdown report
    report = generate_markdown_report(data, timestamp)
    md_path = output_dir / f"diagnostic_baseline_{datetime.now().strftime('%Y%m%d_%H%M%S')}.md"
    with open(md_path, "w", encoding="utf-8") as f:
        f.write(report)

    # Also save as latest.md
    latest_md = output_dir / "diagnostic_baseline_latest.md"
    with open(latest_md, "w", encoding="utf-8") as f:
        f.write(report)

    print(f"\nSaved report: {md_path}")
    print(f"Saved latest: {latest_md}")
    print(f"Saved JSON:   {json_path}")

    # Print summary to console
    print("\n" + "=" * 60)
    print("DIAGNOSTIC BASELINE SUMMARY")
    print("=" * 60)
    print(f"Overall average score: {data['overall']['average_score']:.1f} / 10")
    print(
        f"Worst scenario: {data['scenarios'][0]['scenario']} ({data['scenarios'][0]['average_score']:.1f})"
    )
    print(
        f"Best scenario:  {data['scenarios'][-1]['scenario']} ({data['scenarios'][-1]['average_score']:.1f})"
    )
    print("\nTop 3 hardest to diagnose:")
    for i, s in enumerate(data["scenarios"][:3], 1):
        print(f"  {i}. {s['scenario']} ({s['category']}): {s['average_score']:.1f}/10")
    print("=" * 60)


if __name__ == "__main__":
    main()
