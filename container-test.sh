#!/bin/sh
# container-test - CLI for Linux container runtime verification suites.
# shellcheck shell=sh
set -eu

SCRIPT_NAME=$(basename "$0")

LTPROOT=${LTPROOT:-/opt/ltp}
LTP_SKIPFILE=${LTP_SKIPFILE:-${LTPROOT}/docker-unprivileged.skip}
LTP_WORKERS=${LTP_WORKERS:-4}
OPEN_POSIX_ROOT=${OPEN_POSIX_ROOT:-${LTPROOT}/testcases/open_posix_testsuite}
PJDFSTEST_ROOT=${PJDFSTEST_ROOT:-/opt/pjdfstest}
# Per-test timeout for Open POSIX (seconds). Empty = mode default (quick=30, full=300).
POSIX_TEST_TIMEOUT=${POSIX_TEST_TIMEOUT:-}
POSIX_SKIPFILE=${POSIX_SKIPFILE:-${LTPROOT}/docker-posix.skip}
CONTAINER_TEST_LIBC=${CONTAINER_TEST_LIBC:-unknown}

MODE=quick
RUN_LTP=0
RUN_POSIX=0
RUN_PJDFSTEST=0
SUITE_SELECTED=0
FAILURES=0

usage() {
	cat <<EOF
Usage: ${SCRIPT_NAME} [OPTIONS]

Run Linux container verification suites. With no suite flags, all suites run.
Default mode is a quick pass suitable for smoke verification.

Modes:
  -q, --quick       Quick smoke pass (default)
  -f, --full        Full comprehensive suites

Suites:
  --ltp             Linux Test Project (kirk)
  --posix           Open POSIX Test Suite
  --pjdfstest       pjdfstest filesystem suite

Other:
  -h, --help        Show this help and exit

Examples:
  ${SCRIPT_NAME}
  ${SCRIPT_NAME} --quick --ltp
  ${SCRIPT_NAME} --full --posix --pjdfstest
  ${SCRIPT_NAME} -f --ltp

Environment:
  LTPROOT             LTP install root (default: /opt/ltp)
  LTP_SKIPFILE        kirk skipfile path
  LTP_WORKERS         parallel kirk workers (default: 4)
  OPEN_POSIX_ROOT     Open POSIX tree (default: \$LTPROOT/testcases/open_posix_testsuite)
  PJDFSTEST_ROOT      pjdfstest install root (default: /opt/pjdfstest)
  POSIX_TEST_TIMEOUT  Per-test Open POSIX timeout in seconds (quick default: 30, full: 300)
  POSIX_SKIPFILE      Open POSIX skip list (default: \$LTPROOT/docker-posix.skip)
  CONTAINER_TEST_LIBC Libc flavor of this image (musl or glibc)
  NO_COLOR            Set to disable colored PASSED/SKIPPED/FAILED output
EOF
}

log() {
	printf '%s\n' "$*"
}

err() {
	printf '%s\n' "$*" >&2
}

# Enable ANSI colors when stdout is a TTY and NO_COLOR is unset.
color_init() {
	COLOR_RESET=
	COLOR_PASS=
	COLOR_FAIL=
	COLOR_SKIP=
	if [ -n "${NO_COLOR:-}" ]; then
		return 0
	fi
	if [ -t 1 ]; then
		COLOR_RESET=$(printf '\033[0m')
		COLOR_PASS=$(printf '\033[1;32m')
		COLOR_FAIL=$(printf '\033[1;31m')
		COLOR_SKIP=$(printf '\033[1;33m')
	fi
}

# Print a single test result line with a colored status tag.
# Usage: log_result PASSED|FAILED|SKIPPED "name" ["(0.1s)"]
# Note: avoid the name "status" — callers use that for exit codes.
log_result() {
	lr_label=$1
	lr_name=$2
	lr_detail=${3:-}
	lr_color=
	case ${lr_label} in
	PASSED)
		lr_color=${COLOR_PASS}
		;;
	FAILED)
		lr_color=${COLOR_FAIL}
		;;
	SKIPPED)
		lr_color=${COLOR_SKIP}
		;;
	esac

	if [ -n "${lr_detail}" ]; then
		printf '%s[%s]%s %s %s\n' "${lr_color}" "${lr_label}" "${COLOR_RESET}" "${lr_name}" "${lr_detail}"
	else
		printf '%s[%s]%s %s\n' "${lr_color}" "${lr_label}" "${COLOR_RESET}" "${lr_name}"
	fi
}

mark_result() {
	suite_name=$1
	suite_status=$2

	if [ "${suite_status}" -eq 0 ]; then
		printf '%s==> %s: PASS%s\n' "${COLOR_PASS}" "${suite_name}" "${COLOR_RESET}"
		return 0
	fi

	printf '%s==> %s: FAIL%s\n' "${COLOR_FAIL}" "${suite_name}" "${COLOR_RESET}" >&2
	FAILURES=$((FAILURES + 1))
}

# Format kirk JSON results as [PASSED]/[FAILED]/[SKIPPED] lines.
# Failed/broken/warn entries include indented logs underneath.
ltp_print_results() {
	json_report=$1

	if [ ! -f "${json_report}" ]; then
		err "LTP JSON report not found: ${json_report}"
		return 1
	fi

	python3 - "${json_report}" <<'PY'
import json
import os
import sys

path = sys.argv[1]
with open(path, encoding="utf-8") as fh:
    data = json.load(fh)

use_color = sys.stdout.isatty() and not os.environ.get("NO_COLOR")
colors = {
    "PASSED": "\033[1;32m" if use_color else "",
    "FAILED": "\033[1;31m" if use_color else "",
    "SKIPPED": "\033[1;33m" if use_color else "",
}
reset = "\033[0m" if use_color else ""

status_map = {
    "pass": "PASSED",
    "fail": "FAILED",
    "broken": "FAILED",
    "skip": "SKIPPED",
    "conf": "SKIPPED",
    "warn": "FAILED",
}

failed = 0
for entry in data.get("results", []):
    status = (entry.get("status") or "").lower()
    label = status_map.get(status, "FAILED")
    name = entry.get("test_fqn") or (entry.get("test") or {}).get("command") or "unknown"
    test = entry.get("test") or {}
    duration = float(test.get("duration") or 0.0)
    color = colors.get(label, "")
    print(f"{color}[{label}]{reset} {name} ({duration:.1f}s)")
    if label == "FAILED":
        failed += 1
        log_text = test.get("log") or ""
        if log_text.strip():
            for line in log_text.splitlines():
                print(f"  {line}")
        else:
            print("  (no output captured)")

stats = data.get("stats") or {}
passed = int(stats.get("passed", 0))
skipped = int(stats.get("skipped", 0))
failed_stats = (
    int(stats.get("failed", 0))
    + int(stats.get("broken", 0))
    + int(stats.get("warnings", 0))
)
print(
    f"LTP summary: {passed} passed, {skipped} skipped, {failed_stats} failed"
)
sys.exit(1 if failed_stats else 0)
PY
}

run_ltp() {
	kirk_bin=${LTPROOT}/kirk
	if [ ! -x "${kirk_bin}" ]; then
		err "LTP kirk not found at ${kirk_bin}"
		return 1
	fi
	if ! command -v python3 >/dev/null 2>&1; then
		err "python3 is required to format LTP results"
		return 1
	fi

	suite=smoketest
	if [ "${MODE}" = "full" ]; then
		suite=syscalls
	fi

	json_report=$(mktemp /tmp/kirk-report.XXXXXX)
	# kirk refuses to overwrite an existing report path.
	rm -f "${json_report}"
	kirk_log=$(mktemp /tmp/kirk-log.XXXXXX)

	set -- "${kirk_bin}" -n -o "${json_report}" -w "${LTP_WORKERS}" -f "${suite}"
	if [ -f "${LTP_SKIPFILE}" ]; then
		set -- "${kirk_bin}" -n -S "${LTP_SKIPFILE}" -o "${json_report}" -w "${LTP_WORKERS}" -f "${suite}"
	fi

	status=0
	"$@" >"${kirk_log}" 2>&1 || status=$?

	format_status=0
	if [ -f "${json_report}" ]; then
		ltp_print_results "${json_report}" || format_status=$?
	else
		err "LTP did not produce a JSON report"
		sed 's/^/  /' "${kirk_log}"
		format_status=1
	fi

	rm -f "${json_report}" "${kirk_log}"

	if [ "${format_status}" -ne 0 ] || [ "${status}" -ne 0 ]; then
		return 1
	fi
	return 0
}

posix_now() {
	python3 -c 'import time; print(time.time())'
}

posix_duration() {
	start=$1
	end=$2
	python3 -c "print('{:.1f}'.format(float('${end}') - float('${start}')))"
}

# List .run-test files for one Open POSIX option group (AIO|MEM|MSG|SEM|SIG|THR|TMR|TPS).
posix_list_group_tests() {
	group=$1
	base=${OPEN_POSIX_ROOT}/conformance/interfaces

	if [ ! -d "${base}" ]; then
		return 1
	fi

	# Collect matching interface directories, then find compiled tests under them.
	case ${group} in
	AIO)
		set -- "${base}"/aio_* "${base}"/lio_listio
		;;
	MEM)
		set -- "${base}"/m*lock* "${base}"/m*map "${base}"/shm_*
		;;
	MSG)
		set -- "${base}"/mq_*
		;;
	SEM)
		set -- "${base}"/sem*
		;;
	SIG)
		set -- "${base}"/sig* "${base}"/raise "${base}"/kill "${base}"/killpg \
			"${base}"/pthread_kill "${base}"/pthread_sigmask
		;;
	THR)
		set -- "${base}"/pthread_*
		;;
	TMR)
		set -- "${base}"/time* "${base}"/*time "${base}"/clock* "${base}"/nanosleep
		;;
	TPS)
		set -- "${base}"/*sched*
		;;
	*)
		err "Unknown Open POSIX option group: ${group}"
		return 1
		;;
	esac

	for dir in "$@"; do
		[ -d "${dir}" ] || continue
		find "${dir}" -type f -name '*.run-test'
	done | sort -u
}

# Return 0 if the Open POSIX test basename is listed in POSIX_SKIPFILE.
posix_is_skipped() {
	test_name=$1
	skipfile=${POSIX_SKIPFILE:-}

	if [ -z "${skipfile}" ] || [ ! -f "${skipfile}" ]; then
		return 1
	fi

	# Match whole skipped test names; ignore comments and blanks.
	grep -E "^${test_name}$" "${skipfile}" >/dev/null 2>&1
}

# Run a single Open POSIX .run-test binary with one-line status output.
# Uses openposix t0 (or timeout(1)) so hanging cases like sigpause_4-1 cannot stall the suite.
posix_run_one() {
	test_path=$1
	timeout_secs=$2
	test_name=$(basename "${test_path}" .run-test)
	test_dir=$(dirname "${test_path}")
	test_bin=$(basename "${test_path}")
	t0_bin=${OPEN_POSIX_ROOT}/bin/t0

	log_file=$(mktemp /tmp/posix-test.XXXXXX)
	start=$(posix_now)

	# Capture status with || — `if ! cmd; status=$?` is unreliable (can yield 0).
	status=0
	if [ -x "${t0_bin}" ]; then
		(
			cd "${test_dir}" || exit 1
			"${t0_bin}" "${timeout_secs}" ./"${test_bin}"
		) >"${log_file}" 2>&1 || status=$?
	elif command -v timeout >/dev/null 2>&1; then
		(
			cd "${test_dir}" || exit 1
			timeout "${timeout_secs}" ./"${test_bin}"
		) >"${log_file}" 2>&1 || status=$?
	else
		(
			cd "${test_dir}" || exit 1
			./"${test_bin}"
		) >"${log_file}" 2>&1 || status=$?
	fi

	end=$(posix_now)
	duration=$(posix_duration "${start}" "${end}")

	if [ "${status}" -eq 0 ]; then
		log_result PASSED "${test_name}" "(${duration}s)"
		rm -f "${log_file}"
		return 0
	fi

	log_result FAILED "${test_name}" "(${duration}s)"
	# Print the captured failure output immediately under the failed line.
	if [ -s "${log_file}" ]; then
		sed 's/^/  /' "${log_file}"
	fi
	# t0/timeout typically exit 142 (SIGALRM) or 124 on hang.
	case ${status} in
	124 | 142)
		log "  (timed out after ${timeout_secs}s; test hung)"
		;;
	*)
		if [ ! -s "${log_file}" ]; then
			log "  (no output; exit status ${status})"
		fi
		;;
	esac
	rm -f "${log_file}"
	return 1
}

run_posix() {
	if [ ! -d "${OPEN_POSIX_ROOT}" ]; then
		err "Open POSIX Test Suite not found at ${OPEN_POSIX_ROOT}"
		return 1
	fi

	if ! command -v python3 >/dev/null 2>&1; then
		err "python3 is required to time Open POSIX tests"
		return 1
	fi

	PATH="${OPEN_POSIX_ROOT}/bin:${LTPROOT}/bin:${PATH}"
	export PATH

	groups=SIG
	timeout_secs=${POSIX_TEST_TIMEOUT}
	if [ "${MODE}" = "full" ]; then
		groups="AIO MEM MSG SEM SIG THR TMR TPS"
		if [ -z "${timeout_secs}" ]; then
			timeout_secs=300
		fi
	elif [ -z "${timeout_secs}" ]; then
		# Short enough to unblock known musl hangs (e.g. sigpause_4-1).
		timeout_secs=30
	fi

	failed=0
	total=0
	skipped=0
	log "Open POSIX per-test timeout: ${timeout_secs}s"
	if [ -f "${POSIX_SKIPFILE}" ]; then
		log "Open POSIX skipfile: ${POSIX_SKIPFILE}"
	fi
	# shellcheck disable=SC2086
	for group in ${groups}; do
		log "Open POSIX option group: ${group}"
		list_file=$(mktemp /tmp/posix-list.XXXXXX)
		if ! posix_list_group_tests "${group}" >"${list_file}"; then
			rm -f "${list_file}"
			err "Failed to list tests for group ${group}"
			failed=$((failed + 1))
			continue
		fi

		if [ ! -s "${list_file}" ]; then
			err "No .run-test files found for group ${group} (were tests compiled?)"
			rm -f "${list_file}"
			failed=$((failed + 1))
			continue
		fi

		while IFS= read -r test_path; do
			[ -n "${test_path}" ] || continue
			test_name=$(basename "${test_path}" .run-test)
			if posix_is_skipped "${test_name}"; then
				log_result SKIPPED "${test_name}"
				skipped=$((skipped + 1))
				continue
			fi
			total=$((total + 1))
			if ! posix_run_one "${test_path}" "${timeout_secs}"; then
				failed=$((failed + 1))
			fi
		done <"${list_file}"

		rm -f "${list_file}"
	done

	log "Open POSIX summary: ${total} ran, ${skipped} skipped, ${failed} failed"
	if [ "${failed}" -gt 0 ]; then
		return 1
	fi
	return 0
}

# Run a single pjdfstest .t file with one-line status output.
pjdfs_run_one() {
	test_path=$1
	work_dir=$2
	tests_root=$3

	# Name relative to tests/ (e.g. chmod/00.t).
	test_name=${test_path#"${tests_root}/"}
	log_file=$(mktemp /tmp/pjdfs-test.XXXXXX)
	start=$(posix_now)

	status=0
	(
		cd "${work_dir}" || exit 1
		prove -v "${test_path}"
	) >"${log_file}" 2>&1 || status=$?

	end=$(posix_now)
	duration=$(posix_duration "${start}" "${end}")

	if [ "${status}" -eq 0 ]; then
		log_result PASSED "${test_name}" "(${duration}s)"
		rm -f "${log_file}"
		return 0
	fi

	log_result FAILED "${test_name}" "(${duration}s)"
	if [ -s "${log_file}" ]; then
		sed 's/^/  /' "${log_file}"
	else
		log "  (no output; exit status ${status})"
	fi
	rm -f "${log_file}"
	return 1
}

run_pjdfstest() {
	pjdfstest_bin=${PJDFSTEST_ROOT}/pjdfstest
	tests_dir=${PJDFSTEST_ROOT}/tests

	if [ ! -x "${pjdfstest_bin}" ]; then
		err "pjdfstest binary not found at ${pjdfstest_bin}"
		return 1
	fi
	if [ ! -d "${tests_dir}" ]; then
		err "pjdfstest tests not found at ${tests_dir}"
		return 1
	fi
	if ! command -v prove >/dev/null 2>&1; then
		err "prove (Test::Harness) is required to run pjdfstest"
		return 1
	fi
	if ! command -v python3 >/dev/null 2>&1; then
		err "python3 is required to time pjdfstest cases"
		return 1
	fi

	work_dir=$(mktemp -d /tmp/pjdfstest.XXXXXX)
	PATH="${PJDFSTEST_ROOT}:${PATH}"
	export PATH

	target=${tests_dir}
	if [ "${MODE}" = "quick" ]; then
		# Small representative slice for a fast filesystem smoke pass.
		target=${tests_dir}/chmod
	fi

	list_file=$(mktemp /tmp/pjdfs-list.XXXXXX)
	find "${target}" -type f -name '*.t' | sort >"${list_file}"

	if [ ! -s "${list_file}" ]; then
		err "No pjdfstest .t files found under ${target}"
		rm -f "${list_file}"
		rm -rf "${work_dir}"
		return 1
	fi

	failed=0
	total=0
	while IFS= read -r test_path; do
		[ -n "${test_path}" ] || continue
		total=$((total + 1))
		if ! pjdfs_run_one "${test_path}" "${work_dir}" "${tests_dir}"; then
			failed=$((failed + 1))
		fi
	done <"${list_file}"

	rm -f "${list_file}"
	rm -rf "${work_dir}"

	log "pjdfstest summary: ${total} test(s), ${failed} failed"
	if [ "${failed}" -gt 0 ]; then
		return 1
	fi
	return 0
}

while [ "$#" -gt 0 ]; do
	case $1 in
	-h | --help)
		usage
		exit 0
		;;
	-q | --quick)
		MODE=quick
		;;
	-f | --full)
		MODE=full
		;;
	--ltp)
		RUN_LTP=1
		SUITE_SELECTED=1
		;;
	--posix)
		RUN_POSIX=1
		SUITE_SELECTED=1
		;;
	--pjdfstest)
		RUN_PJDFSTEST=1
		SUITE_SELECTED=1
		;;
	--)
		shift
		break
		;;
	-*)
		err "Unknown option: $1"
		err "Try '${SCRIPT_NAME} -h' for help."
		exit 2
		;;
	*)
		err "Unexpected argument: $1"
		err "Try '${SCRIPT_NAME} -h' for help."
		exit 2
		;;
	esac
	shift
done

if [ "$#" -gt 0 ]; then
	err "Unexpected argument: $1"
	err "Try '${SCRIPT_NAME} -h' for help."
	exit 2
fi

if [ "${SUITE_SELECTED}" -eq 0 ]; then
	RUN_LTP=1
	RUN_POSIX=1
	RUN_PJDFSTEST=1
fi

color_init

log "container-test libc=${CONTAINER_TEST_LIBC} mode=${MODE} ltp=${RUN_LTP} posix=${RUN_POSIX} pjdfstest=${RUN_PJDFSTEST}"

if [ "${RUN_LTP}" -eq 1 ]; then
	log "==> Running Linux Test Project (${MODE})"
	status=0
	run_ltp || status=$?
	mark_result "Linux Test Project" "${status}"
fi
if [ "${RUN_POSIX}" -eq 1 ]; then
	log "==> Running Open POSIX Test Suite (${MODE})"
	status=0
	run_posix || status=$?
	mark_result "Open POSIX Test Suite" "${status}"
fi
if [ "${RUN_PJDFSTEST}" -eq 1 ]; then
	log "==> Running pjdfstest (${MODE})"
	status=0
	run_pjdfstest || status=$?
	mark_result "pjdfstest" "${status}"
fi

if [ "${FAILURES}" -gt 0 ]; then
	err "Completed with ${FAILURES} failed suite(s)."
	exit 1
fi

log "All selected suites completed successfully."
exit 0
