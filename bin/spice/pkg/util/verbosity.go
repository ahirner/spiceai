/*
Copyright 2024 The Spice.ai OSS Authors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

     https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package util

// Verbosity is a struct to handle the complexity of verbosity level from the CLI flags.
// Specifically, it lets us have both `--verbose` and `--very-verbose` flags.
//
// VeryVerbose is a boolean flag to indicate at least 2 levels of verbosity.
// If `VerbosityCount` is greater than 2, it will be ignored, otherwise it will
// be set to 2 if `VeryVerbose` is true.
type Verbosity struct {
	VerbosityCount int
	VeryVerbose    bool
}

func NewVerbosity() *Verbosity {
	return &Verbosity{}
}

func (v *Verbosity) GetLevel() int {
	if v.VerbosityCount > 2 {
		return v.VerbosityCount
	}

	if v.VeryVerbose {
		return 2
	}

	return v.VerbosityCount
}