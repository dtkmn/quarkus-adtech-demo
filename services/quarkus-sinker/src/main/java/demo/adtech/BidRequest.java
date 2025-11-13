package demo.adtech;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import java.util.List;

// This is the POJO from the Kafka topic.
// We must duplicate this from the 'bid-receiver' project.
// (In a monorepo, this would be in a shared 'common-types' module)
// We add @JsonIgnoreProperties to avoid errors if the receiver adds new fields
@JsonIgnoreProperties(ignoreUnknown = true)
public class BidRequest {
    public String id;
    public Site site;
    public App app;
    public Device device;
    public User user;

    public static class Site {
        public String domain;
    }
    public static class App {
        public String bundle;
    }
    public static class Device {
        public String ip;
        public String os;
        public int lmt;
    }
    public static class User {
        public String id;
    }
}