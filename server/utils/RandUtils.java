package server.utils;

import java.util.Random;
import java.util.concurrent.ThreadLocalRandom;

//contins static methods used to generate randomness.
public class RandUtils {

    /**
     * generate random number without a seed.
     * @param minInc minimum number that can be generated, inclusive
     * @param maxInc maximum number that can be generated, inclusive
     * @return random number bewteen minInc and maxInc, inclusive.
     */
    public static int rand(int minInc, int maxInc) {
        return ThreadLocalRandom.current().nextInt(minInc, maxInc + 1);
    }

    /**
     * randomly choose element of array without using a seed, and return it.
     * @param array the provided array
     * @return random element of array
     */
    public static <T> T getRandom(T[] array) {
        return array[rand(0, array.length - 1)];
    }

    /**
     * create perlin noise with provided width, height, and octaveCount, using the random number generator provided.
     * @param width
     * @param height
     * @param random random number generator used to create the noise, to change the perlin noise, change this.
     * @param octaveCount roughly corresponds to the 'spikyness' of the perlin noise. The higher this is, the smoother the perlin noise.
     * @return float[][] of width and height provided, containing perlin noise ranging from 0 to 1.
     */
    public static MultiDimensionalFloatArray generatePerlinNoise(int width, int height, Random random, int octaveCount) {
        MultiDimensionalFloatArray whiteNoise = new MultiDimensionalFloatArray(width, height);
        MultiDimensionalFloatArray[] totalNoise = new MultiDimensionalFloatArray[octaveCount];
        MultiDimensionalFloatArray perlinNoise = new MultiDimensionalFloatArray(width, height);

        float amplitude = 1.0f;
        float totalAmplitude = 0.0f;
        float persistance = 0.5f;

        for (int i = 0; i < width * height; i++) {
            whiteNoise.set(random.nextFloat() % 1, i);
        }

        for (int i = 0; i < octaveCount; i++) {
            totalNoise[i] = perlinNoise(width, height, i, whiteNoise);
        }
        for (int o = octaveCount - 1; o >= 0; o--) {
            amplitude *= persistance;
            totalAmplitude += amplitude;

            for (int i = 0; i < width * height; i++) {
                perlinNoise.set(perlinNoise.get(i) + totalNoise[o].get(i) * amplitude, i);
            }
        }
        for (int i = 0; i < width * height; i++) {
            perlinNoise.set(perlinNoise.get(i) / totalAmplitude);
        }
        return perlinNoise;
    }

    private static MultiDimensionalFloatArray perlinNoise(int width, int height, int octave, MultiDimensionalFloatArray whiteNoise) {
        MultiDimensionalFloatArray result = new MultiDimensionalFloatArray(width, height);

        int samplePeriod = 1 << octave;
        float sampleFrequency = 1.0f / samplePeriod;

        for (int j = 0; j < height; j++) {
            int y1 = (j / samplePeriod) * samplePeriod;
            int y2 = (y1 + samplePeriod) % height;
            float yBlend = (j - y1) * sampleFrequency;

            for (int i = 0; i < width; i++) {
                int x1 = (i / samplePeriod) * samplePeriod;
                int x2 = (x1 + samplePeriod) % width;
                float xBlend = (i - x1) * sampleFrequency;

                float top = lerp(whiteNoise.get(x1, y1), whiteNoise.get(x2, y1), xBlend);

                float bottom = lerp(whiteNoise.get(x1, y2), whiteNoise.get(x2, y2), xBlend);

                result.set(lerp(top, bottom, yBlend), i, j);
            }
        }
        return result;
    }

    private static float lerp(float a, float b, float blend) {
        return a * (1 - blend) + b * blend;
    }
}
