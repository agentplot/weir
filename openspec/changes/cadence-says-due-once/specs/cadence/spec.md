# Spec Delta

## Purpose

Says whether a scheduled thing is due, from its five-field cron
schedule, its last run and now alone, so every host and machine of the
family that asks gets the same answer (59, 60, C.5).

## ADDED Requirements

### Requirement: A thing that has never run is due

Cadence SHALL say due for a thing whose last run is that it never has
(84), whatever its schedule and whatever now is (59).

#### Scenario: First tick of a new schedule

- **WHEN** a thing scheduled `0 9 * * *` has never run and now is
  2026-11-03T08:00 in America/New_York
- **THEN** cadence says due, though 09:00 has not yet come that day

### Requirement: A thing is due when a named time has come since its last run

Cadence SHALL say due exactly when some time its schedule names lies
strictly after the last run and at or before now, and not due otherwise
(59). A named time equal to the last run has not come since it.

#### Scenario: Before the next named time

- **WHEN** a thing scheduled `0 * * * *` last ran at 09:00:00 and now is
  09:59:59 the same day
- **THEN** cadence says not due

#### Scenario: At the next named time

- **WHEN** a thing scheduled `0 * * * *` last ran at 09:00:00 and now is
  10:00:00 the same day
- **THEN** cadence says due

#### Scenario: A run that began on the named time

- **WHEN** a thing scheduled `0 * * * *` last ran at 10:00:00 and now is
  10:00:30 the same day
- **THEN** cadence says not due

#### Scenario: A run that began a fraction before the named time

- **WHEN** a thing scheduled `0 * * * *` last ran at 09:59:59.700 and now
  is 10:00:00.000 the same day
- **THEN** cadence says due

#### Scenario: A last run later than now

- **WHEN** a thing scheduled `0 * * * *` last ran at 11:00:00 and now is
  10:30:00 the same day
- **THEN** cadence says not due

### Requirement: A run missed while the machine slept is caught up once

However many times a schedule named while a machine slept, cadence SHALL
say due once on waking, and once that run is the last run, not due until
the next time the schedule names (60, S13).

#### Scenario: Three hourly times slept through

- **WHEN** a thing scheduled `0 * * * *` last ran at 06:00, the machine
  sleeps until 09:20, and cadence is asked at 09:20
- **THEN** cadence says due, one answer for the 07:00, 08:00 and 09:00
  it missed
- **AND WHEN** the thing then runs at 09:20 and cadence is asked at
  09:21 and at 09:59
- **THEN** cadence says not due both times
- **AND WHEN** cadence is asked at 10:00
- **THEN** cadence says due

### Requirement: Cadence reads the schedule, the last run and now and nothing else

Cadence SHALL answer from the schedule, the last run and now alone, so
two machines given the same three agree (59). Through the clock it SHALL
ask for now and the last run, once each, and for nothing else (85). It
SHALL read no file, network, setting or environment variable to answer.

#### Scenario: Two machines given the same three

- **WHEN** the same schedule, last run and now are given to cadence in
  two processes whose own time zones are Asia/Tokyo and
  America/Los_Angeles
- **THEN** both say the same

#### Scenario: The clock is asked for two things

- **WHEN** cadence is asked whether a thing is due through a clock that
  records what it is asked
- **THEN** the clock was asked for now once and for that thing's last
  run once, and nothing more

### Requirement: The schedule is read in the machine's local time

Cadence SHALL read the times a schedule names in the time zone of the
now it is given (C.5), and the machine's clock SHALL give now in the
machine's local time zone (83; ruling 7 of section 10).

#### Scenario: The same instant in two zones

- **WHEN** a thing scheduled `0 9 * * *` last ran at the instant
  2026-11-02T17:00Z, and now is the instant 2026-11-03T14:30Z
- **THEN** cadence says due when that now is given in Europe/Berlin,
  where it is 15:30 and 09:00 has come since 18:00 the day before, and
  not due when it is given in America/Los_Angeles, where it is 06:30
  and the last run was 09:00 the day before

#### Scenario: The machine's now

- **WHEN** the machine's clock is read in a process whose local time
  zone is Australia/Adelaide
- **THEN** the now it gives is in Australia/Adelaide

### Requirement: A schedule is five-field cron

Cadence SHALL read a schedule of five fields, minute, hour, day of
month, month and day of week, each `*`, a number, a range, a list or a
step over `*` or a range, with month and weekday names. When both day
fields are restricted, a day matching either SHALL do; when one is `*`,
the other alone SHALL decide (C.5).

#### Scenario: Lists, ranges, steps and names

- **WHEN** the schedule `*/15 9-17 * JAN,JUL mon-fri` is read
- **THEN** it names 09:00, 09:15 and onward to 17:45 on weekdays of
  January and July, and no other time

#### Scenario: A weekday with day of month unrestricted

- **WHEN** a thing scheduled `0 9 * * MON` last ran on Monday
  2026-11-02T09:00 and now is Sunday 2026-11-08T23:59
- **THEN** cadence says not due

#### Scenario: Both day fields restricted

- **WHEN** the schedule `0 9 1 * MON` is read
- **THEN** it names 09:00 on the first of each month and on every
  Monday

#### Scenario: Sunday as 0 and as 7

- **WHEN** the schedules `0 9 * * 0` and `0 9 * * 7` are read
- **THEN** both name 09:00 on Sundays alone

### Requirement: A schedule that is not five-field cron is refused

Cadence SHALL refuse, when it is read, a schedule with other than five
fields, a nickname, a seconds or year field, a specifier five-field cron
does not have (`L`, `W`, `#`, `?`, `+`), an unknown name, or a value out
of its field's range, naming the field and why. A refused schedule is
never read as never due.

#### Scenario: Six fields

- **WHEN** the schedule `0 0 9 * * *` is read
- **THEN** it is refused, naming the count of fields

#### Scenario: A nickname

- **WHEN** the schedule `@hourly` is read
- **THEN** it is refused, naming the nickname

#### Scenario: An extension

- **WHEN** the schedule `0 9 L * *` is read
- **THEN** it is refused, naming the day-of-month field and `L`

#### Scenario: A value out of range

- **WHEN** the schedule `60 * * * *` is read
- **THEN** it is refused, naming the minute field and `60`

### Requirement: Daylight-saving changes name each fixed time once

On a day the local clock skips forward, a fixed time the skip removes
SHALL be named at the first minute after it; on a day the local clock
falls back, a fixed time that occurs twice SHALL be named at its first
occurrence alone (C.5, as cron reads local time).

#### Scenario: A time the clock skips

- **WHEN** a thing scheduled `30 2 * * *` in America/New_York last ran
  at 2026-03-07T02:30-05:00 and now is 2026-03-08T03:00-04:00, the
  first minute after that morning's skip from 02:00 to 03:00
- **THEN** cadence says due

#### Scenario: A time the clock repeats

- **WHEN** a thing scheduled `30 1 * * *` in America/New_York last ran
  at 2026-11-01T01:30-04:00, in the first pass of that morning's
  repeated hour, and now is 2026-11-01T01:45-05:00, in the second pass
- **THEN** cadence says not due
