package server.utils;

// contains static methods used to do calculations
public class MathUtils {
    /**
     * returns manhattan distance
     * @param x1
     * @param y1
     * @param x2
     * @param y2 
     * @return manhattan distance between (x1, y1) and (x2, y2)
     */
    public static int manhattan(int x1, int y1, int x2, int y2) {
        return abs(x1 - x2) + abs(x2 - y2);
    }

    /**
     * returns euclidean distance squared
     * @param x1
     * @param y1
     * @param x2
     * @param y2
     * @return square of euclidean distance between (x1, y1) and (x2, y2)
     */
    public static int euclideanSqr(int x1, int y1, int x2, int y2) {
        return (x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2);
    }

    /**
     * returns sign of a
     * @param a
     * @return sign of a (-1 for negatives, 0 for 0, 1 for positives)
     */
    public static int sign(int a) {
        if (a > 0) {
            return 1;
        } else if (a == 0) {
            return 0;
        } else {
            return -1;
        }
    }

    /**
     * return absolute value
     * @param a
     * @return absolute value of a
     */
    public static int abs(int a) {
        return a < 0 ? -a : a;
    }

    /**
     * return min
     * @param a
     * @param b
     * @return minimum value between a and b
     */
    public static int min(int a, int b) {
        return a < b ? a : b;
    }

    /**
     * return max
     * @param a
     * @param b
     * @return maximum value between a and b
     */
    public static int max(int a, int b) {
        return a > b ? a : b;
    }

    /**
     * return a, but bounded between min and max
     * @throws IllegalArgumentException if max < min
     * @param a
     * @param min
     * @param max
     * @return a if a > min && a < max, return max if a >= max, return min if a <= min.
     */
    public static int bound(int a, int min, int max) {
        if(max < min) {
            throw new IllegalArgumentException();
        }
        return max(min, min(max, a));
    }
}