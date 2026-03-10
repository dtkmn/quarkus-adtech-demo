package demo.adtech;

import jakarta.enterprise.context.ApplicationScoped;
import org.eclipse.microprofile.config.inject.ConfigProperty;

@ApplicationScoped
public class BenchmarkSettings {

    @ConfigProperty(name = "benchmark.delivery.mode", defaultValue = "confirm")
    String deliveryMode;

    public boolean isConfirmDeliveryMode() {
        return !"enqueue".equalsIgnoreCase(deliveryMode);
    }
}
