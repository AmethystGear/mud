import java.io.*;
import java.util.*;

public class MudServer {
    // configuration files
    private static final String MOB_FILE = "mobs.txt";
    private static final String BLOCKS_FILE = "blocks.txt";

    // world save file
    private static final String WORLD_SAVE = "world-save.txt";

    public static void main(String[] args) {
        // find all drops, and find the number of mobs.
        Set<String> set = new HashSet<String>();
        Scanner fr = new Scanner(new File(MOB_FILE));
        while (fr.hasNextLine()) {
            String line = fr.nextLine();
            if(line.trim().equals("/begin/")) {
                NUM_MOB_TYPES++;
            } else {
                Scanner tok = new Scanner(line);
                if(tok.hasNext()) {
                    tok.next();
                }
                if(tok.hasNext() && tok.next().equals("drops")) {
                    String[] drops = ScannerUtils.getRemainingInputAsStringArray(tok);
                    for(String drop : drops) {
                        set.add(drop);
                    }
                }
            }
        }
        String [] allDrops = new String[set.size()];
        int j = 0;
        for(String s : set) {
            allDrops[j] = s;
            j++;
        }

        Block.BlockSet blocks = Block.getBlocksFromFile(BLOCKS_FILE);

        boolean makeNewWorld;
        Scanner in = new Scanner(System.in);
        System.out.print("Do you want to load your saved world, or create a new one?(load/create): ");
        String inp = in.nextLine();
        while(!inp.equals("load") && !inp.equals("create")) {
            System.out.print("Please type load or create: ");
            inp = in.nextLine();
        }
        makeNewWorld = inp.equals("create");

        World world;
        if (makeNewWorld) {
            world = new World(RandUtils.rand(0, Integer.MAX_VALUE - 1), NUM_MOB_TYPES, blocks);
        } else {
            world = new World(WORLD_SAVE, NUM_MOB_TYPES, blocks);
        }

        
    }
}