import java.util.Random;
import java.util.concurrent.ThreadLocalRandom;

public class RandUtils {
    public static int rand(int minInc, int maxInc) {
        return ThreadLocalRandom.current().nextInt(minInc, maxInc + 1);
    }

    public static String getRandom(String[] array) {
        return array[rand(0, array.length - 1)];
    }

    public static int getRandom(int[] array) {
        return array[rand(0, array.length - 1)];
    }

    public static Random getXYRand(int x, int y) {
        int hash = x * (int)Math.pow(10, (x + "").length()) + y; 
        return new Random(hash);
    }
}