package server.main;

import java.io.*;
import java.util.*;

import server.objects.instantiables.Block;
import server.objects.Entity;
import server.objects.Instantiable;
import server.objects.Instantiables;
import server.objects.Spawnable;
import server.objects.Entity.EntitySet;
import server.objects.Stats;
import server.utils.MultiDimensionalFloatArray;
import server.utils.MultiDimensionalIntArray;
import server.utils.RandUtils;

public class World {
    public static final int MAP_SIZE = 3000;
    private MultiDimensionalIntArray worldMap;
    private MultiDimensionalIntArray entityMap;

    public final EntitySet entities;

    private final String instantiablesConfigDirectory = "config/instantiables/";

    private int seed;

    public World(String saveFile) throws FileNotFoundException {
        worldMap = new MultiDimensionalIntArray(MAP_SIZE, MAP_SIZE);
        entityMap = new MultiDimensionalIntArray(MAP_SIZE, MAP_SIZE);

        Scanner scan = new Scanner(new File(saveFile));
        seed = scan.nextInt();
        System.out.println(seed);
        readMap(scan, worldMap);
        readMap(scan, entityMap);
        scan.nextLine();
        entities = new Entity.EntitySet();
        entities.add(scan);
        scan.close();
    }

    public World(int seed) throws FileNotFoundException {
        this.seed = seed;
        worldMap = new MultiDimensionalIntArray(MAP_SIZE, MAP_SIZE);
        entityMap = new MultiDimensionalIntArray(MAP_SIZE, MAP_SIZE);

        entities = new Entity.EntitySet();

        File configDir = new File(instantiablesConfigDirectory);
        File[] directoryListing = configDir.listFiles();
        for(File configFile : directoryListing) {
            Stats stat = new Stats();
            String fName = configFile.getName().split("\\.")[0];
            if(!fName.equals("misc")) {
                stat.set("entity type", fName);
            }
            entities.add(new Scanner(configFile), stat);
        }

        Random rand = new Random(seed);
        MultiDimensionalFloatArray perlinNoise = RandUtils.generatePerlinNoise(MAP_SIZE, MAP_SIZE, rand, 10);

        float waterLevel = 0.5f;
        float sandLevel = 0.53f;
        float grassLevel = 0.75f;
        float tallGrassLevel = 0.85f;
        // create map
        for (int i = 0; i < MAP_SIZE * MAP_SIZE; i++) {
            int water = entities.get("water").ID();
            int sand = entities.get("sand").ID();
            int grass = entities.get("grass").ID();
            int tallGrass = entities.get("forest").ID();
            int rock = entities.get("rock").ID();
            int block = 0;
            float height = perlinNoise.get(i);
            if (height < waterLevel) {
                block = water;
            } else if (height >= waterLevel && height < sandLevel) {
                block = sand;
            } else if (height >= sandLevel && height < grassLevel) {
                block = grass;
            } else if (height >= grassLevel && height < tallGrassLevel) {
                block = tallGrass;
            } else {
                block = rock;
            }
            worldMap.set(block, i);
        }

        int numVillages = rand.nextInt(50) + 50;
        for (int i = 0; i < numVillages; i++) {
            int x = rand.nextInt(2000) + 500;
            int y = rand.nextInt(2000) + 500;
            if (!((String) entities.get(worldMap.get(x, y)).getStats().get("name")).contains("water")) {
                spawnVillage(x, y, rand);
            }
        }

        for (int i = 0; i < MAP_SIZE * MAP_SIZE; i++) {
            Entity block = entities.get(worldMap.get(i));
            entityMap.set(-1, i);
            if (!block.getStats().hasProperty("solid")) {
                if (block.getStats().hasVariable("mob spawn chance")) {
                    double mobSpawnChance = (Double)block.getStats().get("mob spawn chance");
                    if (rand.nextDouble() < mobSpawnChance) {
                        entityMap.set(rand.nextInt(entities.size()), i);
                    }
                }
            }
        }
    }

    public void saveTo(String file) throws FileNotFoundException {
        PrintWriter writer = new PrintWriter(new FileOutputStream(new File(file)));
        writer.write(seed + " ");
        saveMap(writer, worldMap);
        saveMap(writer, entityMap);
        writer.write("\n");
        entities.saveTo(writer);
        writer.close();
    }

    private static void saveMap(PrintWriter writer, MultiDimensionalIntArray map) {
        for (int x = 0; x < MAP_SIZE * MAP_SIZE; x++) {
            writer.write(map.get(x) + " ");
        }
    }

    private static void readMap(Scanner scan, MultiDimensionalIntArray map) {
        for (int x = 0; x < MAP_SIZE * MAP_SIZE; x++) {
            map.set(scan.nextInt(), x);
        }
    }

    public Random getXYRand(int x, int y) {
        long hash = x * (int) Math.pow(10, (x + "").length()) + y;
        hash += seed;
        return new Random(hash);
    }

    public Block getBlock(int x, int y) {
        return (Block)clone(getBlockEntity(x, y));
    }

    public Entity getBlockEntity(int x, int y) {
        return entities.get(worldMap.get(x, y));
    }

    public Entity getSpawnableEntity(int x, int y) {
        return entities.get(entityMap.get(x, y));
    }

    public Spawnable getSpawnable(int x, int y) {
        return (Spawnable)clone(getSpawnableEntity(x, y));
    }

    private Object clone(Entity e) {
        String entityType = (String)e.getStats().get("entity type");
        System.out.println(e.getStats());
        Instantiable i = (Instantiable)Instantiables.instantiables.get(entityType);
        return i.create(e);
    }

    public boolean hasEntity(int x, int y) {
        return entityMap.get(x, y) != -1;
    }

    public void removeEntity(int x, int y) {
        entityMap.set(-1, x, y);
    }

    private void spawnVillage(int xOrigin, int yOrigin, Random rand) {
        int floor = entities.get("village floor").ID();
        int villageLength = rand.nextInt(100) + 20;
        int pathSize = rand.nextInt(2) + 3;
        for (int x = xOrigin; x < xOrigin + villageLength; x++) {
            for (int y = yOrigin; y < yOrigin + pathSize; y++) {
                worldMap.set(floor, x, y);
            }
        }
        boolean generateUp = false;
        for (int x = xOrigin + rand.nextInt(3) + 2; x < xOrigin + villageLength; x += (rand.nextInt(5) + 5)) {
            int pathlen = rand.nextInt(7) + 3;
            int hutSize = rand.nextInt(2) + 2;
            if (generateUp) {
                for (int y = yOrigin; y > yOrigin - pathlen; y--) {
                    worldMap.set(floor, x, y);
                }
                spawnHut(x - hutSize, yOrigin - pathlen - hutSize * 2 + 1, hutSize, rand);
            } else {
                for (int y = yOrigin + pathSize; y < yOrigin + pathlen + pathSize; y++) {
                    worldMap.set(floor, x, y);
                }
                spawnHut(x - hutSize, yOrigin + pathlen + pathSize, hutSize, rand);
            }
            generateUp = !generateUp;
        }
    }

    private void spawnHut(int xOrigin, int yOrigin, int size, Random rand) {
        int floor = entities.get("village floor").ID();
        int wall = entities.get("village wall").ID();
        size = size * 2 + 1;
        for (int x = xOrigin; x < xOrigin + size; x++) {
            for (int y = yOrigin; y < yOrigin + size; y++) {
                worldMap.set(floor, x, y);
            }
        }
        for (int x = xOrigin; x < xOrigin + size; x++) {
            if (x - xOrigin != size / 2) {
                worldMap.set(wall, x, yOrigin);
                worldMap.set(wall, x, yOrigin + size - 1);
            }
        }
        for (int y = yOrigin; y < yOrigin + size; y++) {
            worldMap.set(wall, xOrigin, y);
            worldMap.set(wall, xOrigin + size - 1, y);
        }
    }
}
