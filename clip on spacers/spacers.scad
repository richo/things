forkid = 50;
spacement = 10;
spacerwidth = 3;

engravement = str(spacement, "mm");

difference() {
    cylinder(spacement, forkid + spacerwidth, forkid + spacerwidth, true, $fn = 3600);
     cylinder(spacement+1, forkid, forkid, true, $fn = 3600);
    circular_text ();

};




radius = forkid + spacerwidth;
height = spacement-1;
slices = 20;

text_depth = 3;

circumference = 2 * 3.14159 * radius;
slice_width = circumference / slices;

module circular_text () {
   
    union () {
   
        for (i = [0:1:slices]) {
           
            rotate ([0,0,i*(360/slices)]) translate ([0,-radius,0]) intersection () {
               
                translate ([-slice_width/2 - (i*slice_width) ,0 ,-height / 2]) rotate ([90,0,0])
                linear_extrude(text_depth, center = true, convexity = 10)
                text(engravement, size = spacement - 1, font="Courier Sans:style=Bold");
               
                cube ([slice_width+0.1, text_depth+0.1, height], true);
            }
        }
    }
}