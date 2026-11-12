# The team kit

How David, Counsel, Builder and the Oracle work together, with nothing about any one project in it. It's kept so
the team can start the next project the way this one ended up working, not the way it started.

Kept by Counsel, started 2026-11-12. It lives inside the current project for now, so it's backed up with it; copy
the folder out when that project is in the bag. Add to it as we learn: a habit learned the hard way goes here too
once it's clearly not about one project.

## What's in it

| File | What it is |
|---|---|
| `the-team.md` | Who's who, what each one keeps, and how we pass things between us. |
| `working-with-david.md` | David, and how he likes to work: language, pace, decisions, looks and tests. |
| `principles.md` | The design and engineering principles that held up. |
| `habits.md` | Practical habits and warnings: git with two sessions, Windows, Rust, builds, backups, the laptop. |
| `starter/` | Empty versions of the files a new project starts with: `CLAUDE.md`, `master/` and `journal/`. |

## Starting a new project with it

Nothing here is wired in until a project starts. When one does:

1. **Make the project folder** and copy `starter/` into it: `CLAUDE.md`, `master/`, `journal/`. Fill in the
   project's name and a first line of what it is; leave the rest for the first talk with David.
2. **Give every session the team's ways.** Either copy the four files' contents into the user-level
   `C:\Users\David\.claude\CLAUDE.md`, which every session in every folder reads, or point the new project's
   `CLAUDE.md` at this kit. The user-level file is the better home once the old project is done.
3. **Give each role its sheet.** A session told "you're Counsel" (or Builder, or the Oracle) reads its part of
   `the-team.md` first. The roles can become personal skills later (`C:\Users\David\.claude\skills\<role>\SKILL.md`),
   so `/counsel`, `/builder` and `/oracle` start a session in role. The Oracle can also be an agent definition with
   read-only tools, so it can't edit by its tools, not just by promise.
4. **Memory starts empty.** The memory folder is filed by the folder a session opens in, so a new project's is
   empty, and the old project's habits stay with the old project. That's why the general ones are kept here.
5. **Back it up from day one:** a private GitHub repository and a dated copy on the USB drive (`habits.md`).
