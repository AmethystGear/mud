package server.utils;

import java.util.Random;
import java.util.concurrent.ThreadLocalRandom;

public class RandUtils {

    // seedless random value
    public static int rand(int minInc, int maxInc) {
        return ThreadLocalRandom.current().nextInt(minInc, maxInc + 1);
    }

    // seedless random selection from array
    public static String getRandom(String[] array) {
        return array[rand(0, array.length - 1)];
    }

    public static int getRandom(int[] array) {
        return array[rand(0, array.length - 1)];
    }

    public static float[][] generatePerlinNoise(int width, int height, Random random, int octaveCount) {
        float[][] whiteNoise = new float[width][height];
        float[][][] totalNoise = new float[octaveCount][][];
        float[][] perlinNoise = new float[width][height];
        float amplitude = 1.0f;
        float totalAmplitude = 0.0f;
        float persistance = 0.5f;

        for (int i = 0; i < width; i++) {
            for (int j = 0; j < height; j++) {
                whiteNoise[i][j] = random.nextFloat() % 1;
            }
        }
        for (int i = 0; i < octaveCount; i++) {
            totalNoise[i] = perlinNoise(width, height, i, whiteNoise);
        }
        for (int o = octaveCount - 1; o >= 0; o--) {
            amplitude *= persistance;
            totalAmplitude += amplitude;

            for (int i = 0; i < width; i++) {
                for (int j = 0; j < height; j++) {
                    perlinNoise[i][j] += totalNoise[o][i][j] * amplitude;
                }
            }
        }
        for (int i = 0; i < width; i++) {
            for (int j = 0; j < height; j++) {
                perlinNoise[i][j] /= totalAmplitude;
            }
        }
        return perlinNoise;
    }

    private static float[][] perlinNoise(int width, int height, int octave, float[][] whiteNoise) {
        float[][] result = new float[width][height];

        int samplePeriod = 1 << octave;
        float sampleFrequency = 1.0f / samplePeriod;

        for (int i = 0; i < width; i++) {
            int x1 = (i / samplePeriod) * samplePeriod;
            int x2 = (x1 + samplePeriod) % width;
            float xBlend = (i - x1) * sampleFrequency;

            for (int j = 0; j < height; j++) {
                int y1 = (j / samplePeriod) * samplePeriod;
                int y2 = (y1 + samplePeriod) % height;
                float yBlend = (j - y1) * sampleFrequency;

                float top = lerp(whiteNoise[x1][y1], whiteNoise[x2][y1], xBlend);

                float bottom = lerp(whiteNoise[x1][y2], whiteNoise[x2][y2], xBlend);

                result[i][j] = lerp(top, bottom, yBlend);
            }
        }
        return result;
    }

    private static float lerp(float a, float b, float blend) {
        return a * (1 - blend) + b * blend;
    }
}