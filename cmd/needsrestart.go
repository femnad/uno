package cmd

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"regexp"
	"slices"
	"strconv"
	"strings"

	marecmd "github.com/femnad/mare/cmd"
)

var (
	debianKernelPkgRegex = regexp.MustCompile(`linux-image-([0-9]+\.[0-9]+\.[0-9]+)`)
	kernelVersionRegex   = regexp.MustCompile(`BOOT_IMAGE=(?:\([a-z0-9]+,[a-z0-9]+\))?/vmlinuz-([0-9]+\.[0-9]+\.[0-9]+(-[0-9]+)?)`)
	ubuntuKernelPkgRegex = regexp.MustCompile(`linux-image-([0-9]+\.[0-9]+\.[0-9]+-[0-9]+)`)
)

func getOsId() (string, error) {
	fd, err := os.Open("/etc/os-release")
	if err != nil {
		return "", err
	}
	defer fd.Close()

	scanner := bufio.NewScanner(fd)
	for scanner.Scan() {
		line := scanner.Text()
		if !strings.HasPrefix(line, "ID=") {
			continue
		}

		fields := strings.Split(line, "=")
		if len(fields) != 2 {
			return "", fmt.Errorf("unable to extract OS ID from line %s", line)
		}

		return fields[1], nil
	}

	return "", fmt.Errorf("unable to determine the OS ID")
}

func aptKernelPackages(regex regexp.Regexp) ([]string, error) {
	var pkgs []string
	out, err := marecmd.RunFmtErr(marecmd.Input{Command: "dpkg --list"})
	if err != nil {
		return pkgs, err
	}

	lines := strings.Split(out.Stdout, "\n")
	var versions []string
	for _, line := range lines[5:] {
		fields := strings.Split(line, " ")
		if len(fields) != 2 {
			return pkgs, fmt.Errorf("unable to determine package from line %s", line)
		}

		matches := regex.FindStringSubmatch(fields[1])
		if len(matches) > 0 {
			versions = append(versions, matches[0])
		}
	}

	return versions, nil
}

func dnfKernelPackages() ([]string, error) {
	var pkgs []string
	out, err := marecmd.RunFmtErr(marecmd.Input{Command: "dnf list --installed kernel"})
	if err != nil {
		return pkgs, err
	}

	lines := strings.Split(out.Stdout, "\n")
	for _, line := range lines[1:] {
		if line == "" {
			continue
		}

		re := regexp.MustCompile(`\s+`)
		fields := re.Split(line, -1)
		if len(fields) != 3 {
			return pkgs, fmt.Errorf("unable to determine version from line `%s`", line)
		}

		version := fields[1]
		dotIndex := strings.LastIndex(version, ".")
		if dotIndex == -1 {
			return pkgs, fmt.Errorf("unable to determine dot index of line `%s`", line)
		}

		pkgs = append(pkgs, version[:dotIndex])
	}

	return pkgs, nil
}

func listKernelPackages() ([]string, error) {
	var pkgs []string
	osId, err := getOsId()
	if err != nil {
		return pkgs, err
	}

	switch osId {
	case "debian":
		pkgs, err = aptKernelPackages(*debianKernelPkgRegex)
		if err != nil {
			return pkgs, err
		}
	case "fedora":
		pkgs, err = dnfKernelPackages()
		if err != nil {
			return pkgs, err
		}
	case "ubuntu":
		pkgs, err = aptKernelPackages(*ubuntuKernelPkgRegex)
		if err != nil {
			return pkgs, err
		}
	}

	return pkgs, nil
}

func compareVersions(a, b string) int {
	splitBy := "."
	if strings.Contains(a, "-") {
		splitBy = `\.|-`
	}
	re := regexp.MustCompile(splitBy)

	aFields := re.Split(a, -1)
	bFields := re.Split(b, -1)
	fieldLen := len(aFields)

	for i, aField := range aFields {
		aInt, _ := strconv.Atoi(aField)
		bField := bFields[i]
		bInt, _ := strconv.Atoi(bField)

		if aInt == bInt {
			if i == fieldLen {
				return 0
			}

			continue
		}

		if aInt < bInt {
			return -1
		}

		return 1
	}

	return 0
}

func mostRecentInstalledKernel() (string, error) {
	pkgs, err := listKernelPackages()
	if err != nil {
		return "", err
	}

	if len(pkgs) == 0 {
		return "", fmt.Errorf("unable to find any kernel packages")
	}

	slices.SortFunc(pkgs, compareVersions)
	return pkgs[len(pkgs)-1], nil
}

func getRunningKernel() (string, error) {
	fd, err := os.Open("/proc/cmdline")
	if err != nil {
		return "", err
	}
	defer fd.Close()

	scanner := bufio.NewScanner(fd)
	for scanner.Scan() {
		line := scanner.Text()
		matches := kernelVersionRegex.FindStringSubmatch(line)
		if len(matches) == 0 {
			continue
		}

		return matches[1], nil
	}

	return "", fmt.Errorf("unable to determine running kernel")
}

func checkNeedsRestart() (int, error) {
	var code int
	runningKernel, err := getRunningKernel()
	if err != nil {
		return code, err
	}

	mostRecent, err := mostRecentInstalledKernel()
	if err != nil {
		return code, err
	}

	if runningKernel == mostRecent {
		code = 1
	}

	return code, nil
}

func needsRestart() {
	code, err := checkNeedsRestart()
	if err != nil {
		log.Fatalf("error checking if restart is needed: %v", err)
	}

	os.Exit(code)
}
