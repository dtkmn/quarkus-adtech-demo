package models

// BidRequest is the Go struct that mirrors the Java BidRequest POJO.
// We use pointers (*Site, *App, *Device) for fields that can be null,
// which is crucial for our validation logic.
type BidRequest struct {
	ID     string  `json:"id"`
	Site   *Site   `json:"site"`
	App    *App    `json:"app"`
	Device *Device `json:"device"`
	User   *User   `json:"user"`
}

type Site struct {
	Domain string `json:"domain"`
}
type App struct {
	Bundle string `json:"bundle"`
}
type Device struct {
	IP  string `json:"ip"`
	OS  string `json:"os"`
	LMT int    `json:"lmt"` // Limit Ad Tracking
}
type User struct {
	ID string `json:"id"`
}
