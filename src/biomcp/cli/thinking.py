"""CLI commands for sequential thinking functionality."""

import json
from typing import Annotated, Optional

import typer
from rich.console import Console
from rich.table import Table

from ..thinking import sequential

console = Console()
thinking_app = typer.Typer(help="Sequential thinking commands")


@thinking_app.command("add")
def add_thought(
    thought: Annotated[str, typer.Argument(help="The thinking step content")],
    thought_number: Annotated[int, typer.Option("--number", "-n", help="Thought number in sequence")] = 1,
    total_thoughts: Annotated[int, typer.Option("--total", "-t", help="Total estimated thoughts needed")] = 1,
    next_needed: Annotated[bool, typer.Option("--next/--no-next", help="Whether another thought is needed")] = False,
    revision: Annotated[bool, typer.Option("--revision", "-r", help="Mark as revision of previous thought")] = False,
    revises_thought: Annotated[Optional[int], typer.Option("--revises", help="Which thought number is being revised")] = None,
    branch_from: Annotated[Optional[int], typer.Option("--branch-from", help="Thought number to branch from")] = None,
    branch_id: Annotated[Optional[str], typer.Option("--branch-id", help="Branch identifier")] = None,
    need_more: Annotated[Optional[bool], typer.Option("--need-more", help="Need more thoughts beyond total")] = None,
):
    """Add a new thought to the sequential thinking process."""

    # Validate inputs
    if revision and not revises_thought:
        console.print("[red]Error: --revises must be specified when using --revision[/red]")
        raise typer.Exit(1)

    if branch_from and not branch_id:
        console.print("[red]Error: --branch-id must be specified when using --branch-from[/red]")
        raise typer.Exit(1)

    # Create thought entry
    entry = {
        "thought": thought,
        "thoughtNumber": thought_number,
        "totalThoughts": total_thoughts,
        "nextThoughtNeeded": next_needed,
        "isRevision": revision,
        "revisesThought": revises_thought,
        "branchFromThought": branch_from,
        "branchId": branch_id,
        "needsMoreThoughts": need_more,
        "timestamp": sequential.get_current_timestamp()
    }

    # Add to appropriate storage
    if branch_id:
        sequential.add_thought_to_branch(entry)
        console.print(f"[green]Added thought {thought_number} to branch '{branch_id}'[/green]")
    else:
        sequential.add_thought_to_history(entry)
        if revision:
            console.print(f"[green]Revised thought {revises_thought} (now thought {thought_number})[/green]")
        else:
            console.print(f"[green]Added thought {thought_number} to main sequence[/green]")

    # Show progress
    progress_msg = f"Progress: {thought_number}/{total_thoughts} thoughts"
    if need_more:
        progress_msg += " (may need more)"

    next_msg = "Next thought needed" if next_needed else "Thinking sequence complete"
    console.print(f"[blue]{progress_msg}. {next_msg}.[/blue]")


@thinking_app.command("history")
def show_history(
    json_output: Annotated[bool, typer.Option("--json", help="Output in JSON format")] = False,
):
    """Show the complete thought history."""

    if not sequential.thought_history:
        console.print("[yellow]No thoughts recorded yet.[/yellow]")
        return

    if json_output:
        console.print(json.dumps(sequential.thought_history, indent=2))
        return

    table = Table(title="Thought History")
    table.add_column("Thought #", style="cyan")
    table.add_column("Content", style="white")
    table.add_column("Total", style="green")
    table.add_column("Next?", style="blue")
    table.add_column("Revision?", style="magenta")
    table.add_column("Branch", style="yellow")
    table.add_column("Timestamp", style="dim")

    for thought in sequential.thought_history:
        table.add_row(
            str(thought["thoughtNumber"]),
            thought["thought"][:80] + "..." if len(thought["thought"]) > 80 else thought["thought"],
            str(thought["totalThoughts"]),
            "Yes" if thought["nextThoughtNeeded"] else "No",
            "Yes" if thought.get("isRevision") else "No",
            thought.get("branchId", ""),
            thought["timestamp"][:19]  # Remove microseconds
        )

    console.print(table)


@thinking_app.command("branches")
def show_branches(
    branch_id: Annotated[Optional[str], typer.Argument(help="Specific branch ID to show")] = None,
    json_output: Annotated[bool, typer.Option("--json", help="Output in JSON format")] = False,
):
    """Show branch information or specific branch thoughts."""

    if not sequential.thought_branches:
        console.print("[yellow]No branches created yet.[/yellow]")
        return

    if branch_id:
        # Show specific branch
        if branch_id not in sequential.thought_branches:
            console.print(f"[red]Branch '{branch_id}' not found.[/red]")
            return

        branch_thoughts = sequential.thought_branches[branch_id]

        if json_output:
            console.print(json.dumps(branch_thoughts, indent=2))
            return

        table = Table(title=f"Branch: {branch_id}")
        table.add_column("Thought #", style="cyan")
        table.add_column("Content", style="white")
        table.add_column("Branched From", style="green")
        table.add_column("Timestamp", style="dim")

        for thought in branch_thoughts:
            table.add_row(
                str(thought["thoughtNumber"]),
                thought["thought"][:80] + "..." if len(thought["thought"]) > 80 else thought["thought"],
                str(thought.get("branchFromThought", "")),
                thought["timestamp"][:19]
            )

        console.print(table)
    else:
        # Show all branches summary
        if json_output:
            console.print(json.dumps(sequential.thought_branches, indent=2))
            return

        table = Table(title="All Branches")
        table.add_column("Branch ID", style="cyan")
        table.add_column("Thought Count", style="green")
        table.add_column("Last Thought", style="white")

        for branch_id, thoughts in sequential.thought_branches.items():
            last_thought = thoughts[-1]["thought"] if thoughts else ""
            table.add_row(
                branch_id,
                str(len(thoughts)),
                last_thought[:60] + "..." if len(last_thought) > 60 else last_thought
            )

        console.print(table)


@thinking_app.command("summary")
def show_summary():
    """Show a summary of the current thinking process."""

    total_thoughts = len(sequential.thought_history)
    total_branches = len(sequential.thought_branches)

    table = Table(title="Thinking Process Summary")
    table.add_column("Metric", style="cyan")
    table.add_column("Value", style="green")

    table.add_row("Total Thoughts", str(total_thoughts))
    table.add_row("Active Branches", str(total_branches))

    if sequential.thought_branches:
        table.add_row("Branch Names", ", ".join(sequential.thought_branches.keys()))

    if sequential.thought_history:
        last_thought = sequential.thought_history[-1]["thought"]
        table.add_row("Last Thought", last_thought[:100] + "..." if len(last_thought) > 100 else last_thought)

    console.print(table)


@thinking_app.command("clear")
def clear_history(
    confirm: Annotated[bool, typer.Option("--confirm", help="Confirm clearing all history")] = False,
):
    """Clear all thinking history and branches."""

    if not confirm:
        console.print("[yellow]Use --confirm to actually clear the thinking history.[/yellow]")
        return

    old_thought_count = len(sequential.thought_history)
    old_branch_count = len(sequential.thought_branches)

    sequential.thought_history.clear()
    sequential.thought_branches.clear()

    console.print(f"[green]Cleared {old_thought_count} thoughts and {old_branch_count} branches.[/green]")


@thinking_app.command("guide")
def show_guide():
    """Show guidance on using sequential thinking effectively."""

    console.print("[bold cyan]Sequential Thinking Guide[/bold cyan]\n")

    console.print("[bold]Basic Usage:[/bold]")
    console.print("  biomcp thinking add 'Analyze the problem requirements' --number 1 --total 5 --next")
    console.print()

    console.print("[bold]Revision:[/bold]")
    console.print("  biomcp thinking add 'Better analysis of requirements' --number 1 --revision --revises 1")
    console.print()

    console.print("[bold]Branching:[/bold]")
    console.print("  biomcp thinking add 'Try machine learning approach' --number 3 --branch-from 2 --branch-id ml-approach")
    console.print()

    console.print("[bold]View History:[/bold]")
    console.print("  biomcp thinking history")
    console.print("  biomcp thinking branches")
    console.print("  biomcp thinking summary")
    console.print()

    console.print("[bold]Best Practices:[/bold]")
    console.print("  • Start with problem analysis")
    console.print("  • Break down complex problems into steps")
    console.print("  • Use branches to explore alternatives")
    console.print("  • Revise thoughts when you gain new insights")
    console.print("  • Keep thoughts focused and actionable")


if __name__ == "__main__":
    thinking_app()
