package envutil

import (
	"fmt"
	"os"
)

// Gets env variable
func GetEnv(key string) (string, bool) {
	return os.LookupEnv(key)
}

// Tries to get env variable, panics if not found
func GetEnvX(key string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	panic(fmt.Sprintf("env variable %s not found", key))
}

// Tries to get env variable, returns the fallback if not found
func GetEnvWithFallback(key, fallback string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	return fallback
}
