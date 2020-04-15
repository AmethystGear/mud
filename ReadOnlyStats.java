import java.io.FileNotFoundException;

// Wrapper for the stats class that limits functionality.
public class ReadOnlyStats {
    private Stats stats;
    public ReadOnlyStats(Stats stats) {
        this.stats = stats;
    }
    public Object get(String stat) {
        return stats.get(stat);
    }
    public String toString() {
        return stats.toString();
    }
    public void saveTo(String file) throws FileNotFoundException {
        stats.saveTo(file);
    }
}