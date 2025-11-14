package demo.adtech;

import io.quarkus.hibernate.orm.panache.PanacheEntity;
import jakarta.persistence.Entity;
import jakarta.persistence.Table;
import java.time.Instant;

// This is the Panache Entity that will be saved to Postgres.
@Entity
@Table(name = "bid_records")
public class BidRecord extends PanacheEntity {

    // We get a 'long id' primary key for free from PanacheEntity

    public String bidRequestId; // The 'id' from the BidRequest
    public String domain;
    public String appBundle;
    public String ip;
    public String os;
    public boolean limitAdTracking;
    public Instant processedAt;

    // Default constructor required for Hibernate
    public BidRecord() {}

    /**
     * Helper constructor to map from the Kafka object to the DB object.
     */
    public BidRecord(BidRequest request) {
        this.bidRequestId = request.id;

        if (request.site != null) {
            this.domain = request.site.domain;
        }
        if (request.app != null) {
            this.appBundle = request.app.bundle;
        }
        if (request.device != null) {
            this.ip = request.device.ip;
            this.os = request.device.os;
            this.limitAdTracking = (request.device.lmt == 1);
        }

        this.processedAt = Instant.now();
    }
}