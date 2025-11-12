package demo.adtech;

import java.util.List;

public class BidRequest {
    public String id;
    public List<Impression> imp;
    public Site site;
    public App app;
    public Device device;
    public User user;

    public static class Impression {
        public String id;
        public Banner banner;
        public Video video;
        public double bidfloor; // Minimum bid price allowed
        public String bidfloorcur = "USD";
    }

    public static class Banner {
        public int w;
        public int h;
        public int pos;
    }

    public static class Video {
        public List<String> mimes;
        public int minduration;
        public int maxduration;
    }

    public static class Site {
        public String id;
        public String domain;
        public List<String> cat; // IAB Categories
    }

    public static class App {
        public String id;
        public String name;
        public String bundle; // e.g., com.rovio.angrybirds
    }

    public static class Device {
        public String ua; // User Agent
        public String ip;
        public String geo; // Simplified
        public String os;
        public String ifa; // ID for Advertisers (crucial for mobile tracking)
        public int lmt; // Limit Ad Tracking (0 or 1)
    }

    public static class User {
        public String id;
        public String buyeruid;
    }
}
