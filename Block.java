import java.io.*;
import java.util.*;

public class Block {
    public final int BLOCK_ID;
    public final ReadOnlyStats STATS;

    public Block(Stats stats, int blockID) {
        this.STATS = new ReadOnlyStats(stats);
        this.BLOCK_ID = blockID;
    }

    public static BlockSet getBlocksFromFile(String file) {
        try {
            Scanner s = new Scanner(new File(file));
            BlockSet blockSet = new BlockSet();

            int blockID = 0;
            while(s.hasNextLine()) {
                try {
                    Stats stats = new Stats(s);
                    blockSet.addBlock(new Block(stats, blockID));
                    blockID++;
                } catch(IllegalArgumentException e) {
                    return blockSet;
                }
            }
            return blockSet;           
        } catch (IOException e) {
            return null;
        }
    }

    public static class BlockSet {
        private HashMap<String, Block> nameToBlock;
        private HashMap<Integer, Block> blockIDtoBlock;

        public BlockSet() {
            nameToBlock = new HashMap<String, Block>();
            blockIDtoBlock = new HashMap<Integer, Block>();
        }

        private void addBlock(Block b) {
            nameToBlock.put((String)b.STATS.get("name"), b);
            blockIDtoBlock.put(b.BLOCK_ID, b);
        }

        public Block getBlock(int ID) {
            return blockIDtoBlock.get(ID);
        }

        public Block getBlock(String name) {
            return nameToBlock.get(name);
        }
    }
}