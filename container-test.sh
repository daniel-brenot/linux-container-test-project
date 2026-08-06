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
EOF
}

log() {
	printf '%s\n' "$*"
}

err() {
	printf '%s\n' "$*" >&2
}

mark_result() {
	suite_name=$1
	status=$2

	if [ "${status}" -eq 0 ]; then
		log "==> ${suite_name}: PASS"
		return 0
	fi

	err "==> ${suite_name}: FAIL"
	FAILURES=$((FAILURES + 1))
}

run_ltp() {
	kirk_bin=${LTPROOT}/kirk
	if [ ! -x "${kirk_bin}" ]; then
		err "LTP kirk not found at ${kirk_bin}"
		return 1
	fi

	suite=smoketest
	if [ "${MODE}" = "full" ]; then
		suite=syscalls
	fi

	if [ -f "${LTP_SKIPFILE}" ]; then
		"${kirk_bin}" -S "${LTP_SKIPFILE}" -w "${LTP_WORKERS}" -f "${suite}"
		return $?
	fi

	"${kirk_bin}" -w "${LTP_WORKERS}" -f "${suite}"
}

run_posix() {
	posix_runner=${LTPROOT}/bin/run-posix-option-group-test.sh
	posix_all=${LTPROOT}/bin/run-all-posix-option-group-tests.sh

	if [ ! -d "${OPEN_POSIX_ROOT}" ]; then
		err "Open POSIX Test Suite not found at ${OPEN_POSIX_ROOT}"
		return 1
	fi

	# Ensure helpers from the Open POSIX tree are discoverable.
	PATH="${OPEN_POSIX_ROOT}/bin:${LTPROOT}/bin:${PATH}"
	export PATH

	if [ "${MODE}" = "full" ]; then
		if [ -x "${posix_all}" ]; then
			"${posix_all}"
			return $?
		fi
		if [ -x "${OPEN_POSIX_ROOT}/bin/run-all-posix-option-group-tests.sh" ]; then
			"${OPEN_POSIX_ROOT}/bin/run-all-posix-option-group-tests.sh"
			return $?
		fi
		err "Open POSIX full runner not found"
		return 1
	fi

	# Quick: one focused option group (signals).
	if [ -x "${posix_runner}" ]; then
		"${posix_runner}" SIG
		return $?
	fi
	if [ -x "${OPEN_POSIX_ROOT}/bin/run-posix-option-group-test.sh" ]; then
		"${OPEN_POSIX_ROOT}/bin/run-posix-option-group-test.sh" SIG
		return $?
	fi

	err "Open POSIX option-group runner not found"
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

	work_dir=$(mktemp -d /tmp/pjdfstest.XXXXXX)
	PATH="${PJDFSTEST_ROOT}:${PATH}"
	export PATH

	target=${tests_dir}
	if [ "${MODE}" = "quick" ]; then
		# Small representative slice for a fast filesystem smoke pass.
		target=${tests_dir}/chmod
	fi

	status=0
	if ! (
		cd "${work_dir}" || exit 1
		prove -rv "${target}"
	); then
		status=1
	fi

	rm -rf "${work_dir}"
	return "${status}"
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

log "container-test mode=${MODE} ltp=${RUN_LTP} posix=${RUN_POSIX} pjdfstest=${RUN_PJDFSTEST}"

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