package cmd

import (
	"os"

	"github.com/spf13/cobra"
)

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "uno",
	Short: "Do everything",
	Long:  "Do most of the things",
}

var needsRestartCmd = &cobra.Command{
	Use:   "needs-restart",
	Short: "Check if restart is neeeded",
	Run: func(cmd *cobra.Command, args []string) {
		needsRestart()
	},
}

// Execute adds all child commands to the root command and sets flags appropriately.
func Execute() {
	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

func init() {
	rootCmd.AddCommand(needsRestartCmd)
}
