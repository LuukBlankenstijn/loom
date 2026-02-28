package domain

import (
	"errors"

	"github.com/google/uuid"
)

var ErrAlreadyRegistered = errors.New("station already registered")

type Hub interface {
	Register(stationIp string) (<-chan StationHubEvent, func(), error)

	// Notify notify station, optionally filtering by ip
	// This method does not guarantee delivery, if some channel is full the message is dropped
	Send(event StationHubEvent, stationIp ...string)

	// Tries to set the login status of some station. only works if the station is registered
	SetLoginStatus(stationIp string, loggedIn bool)
}

type StationHubEvent interface {
	isStationHubEvent()
}

type SetContestUrl struct {
	Url string
}

type SetWallpaperSource struct {
	Source string
}

type Login struct{}

type Logout struct{}

type LoginWithCredentials struct {
	Username string
	Password string
}

type CustomCommand struct {
	Id      uuid.UUID
	Command string
}

// marker method to implement the Interface
func (SetContestUrl) isStationHubEvent() {}

func (SetWallpaperSource) isStationHubEvent() {}

func (Login) isStationHubEvent() {}

func (Logout) isStationHubEvent() {}

func (LoginWithCredentials) isStationHubEvent() {}

func (CustomCommand) isStationHubEvent() {}
