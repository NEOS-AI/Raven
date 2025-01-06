package log

import (
	"fmt"
	"log"
	"os"
	"path/filepath"
)

var RavenLogger Logger = Logger{}

type Logger struct {
	generalLogger *log.Logger
	errorLogger   *log.Logger
}

const BaseLogPath = "./.log"

// Logger has method "initLoggers", which initialize loggers with basic settings
func (l *Logger) InitLoggers() {
	absPath, err := filepath.Abs(BaseLogPath)
	if err != nil {
		fmt.Println("Error reading given path:", err)
	}
	if _, err := os.Stat(absPath); os.IsNotExist(err) {
		os.MkdirAll(absPath, 0755)
	}

	generalLog, err := os.OpenFile(absPath+"/general-log.log", os.O_RDWR|os.O_CREATE|os.O_APPEND, 0666)
	if err != nil {
		fmt.Println("Error opening file:", err)
		os.Exit(1)
	}
	errorLog, err := os.OpenFile(absPath+"/error-log.log", os.O_RDWR|os.O_CREATE|os.O_APPEND, 0666)
	if err != nil {
		fmt.Println("Error opening file:", err)
		os.Exit(1)
	}

	l.generalLogger = log.New(generalLog, "General Logger:\t", log.Ldate|log.Ltime|log.Lshortfile)
	l.errorLogger = log.New(errorLog, "Error Logger:\t", log.Ldate|log.Ltime|log.Lshortfile)
}

func (l Logger) LogDebug(msg string) {
	l.generalLogger.Printf("[Debug]: %s", msg)
}

func (l Logger) LogInfo(msg string) {
	l.generalLogger.Printf("[Info]: %s", msg)
}

func (l Logger) LogWarning(msg string) {
	l.generalLogger.Printf("[Warn]: %s", msg)
	l.errorLogger.Printf("[Warn]: %s", msg)
}

func (l Logger) LogError(msg string) {
	l.generalLogger.Printf("[Error]: %s", msg)
	l.errorLogger.Printf("[Error]: %s", msg)
}

// LogCritical exits the application, since the "critical" error might affect to application critically
func (l Logger) LogCritical(msg string) {
	l.generalLogger.Printf("[Critical]: %s", msg)
	l.errorLogger.Fatalf("[Critical]: %s", msg)
}
