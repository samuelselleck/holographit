# This script to be run after running cargo test

# generate .gif files for multi-svg outputs
ffmpeg -y -f image2 -framerate 15 -i rect-%03d.svg rectangle-animation.gif
ffmpeg -y -f image2 -framerate 15 -i test4-%03d.svg test4-animation.gif
ffmpeg -y -f image2 -framerate 15 -i icosahedron-%03d.svg icosahedron-animation.gif

# Remove intermediate multi-svg outputs
rm test4-0* rect-0* icosahedron-0*

# Look at it all!
firefox rectangle-animation.gif test4-animation.gif icosahedron-animation.gif icosahedron-anim.svg test4-anim.svg rect-anim.svg

echo "Look at all outputs in firefox and ensure they are correct!"
