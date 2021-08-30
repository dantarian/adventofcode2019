package cmd

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
	"pencethren.org/aoc2019/file"
	"pencethren.org/aoc2019/fuel"
)

func init() {
	rootCmd.AddCommand(fuelCmd)
}

var fuelCmd = &cobra.Command{
	Use:   "fuel <file>",
	Short: "Calculate fuel",
	Long:  "Calculate the fuel necessary to launch the ship.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		moduleMasses, err := file.IntLines(args[0])
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error loading file: %v\n", err)
			os.Exit(1)
		}

		ship := fuel.NewShip(moduleMasses[:])
		if !part2 {
			fmt.Printf("Fuel needed: %v\n", ship.Fuel())
		} else {
			fmt.Printf("Fuel needed: %v\n", ship.CompoundFuel())
		}
	},
}
