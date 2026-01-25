package envutil

import (
	"fmt"
	"os"
)

func GetEnv(key string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	panic(fmt.Sprintf("env variable %s not found", key))
}

func GetEnvWithFallback(key, fallback string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	return fallback
}
