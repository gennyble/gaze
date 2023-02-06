I cannot afford to shoot film nor can I afford the three filters I'd need to shoot trichrome at all, but it interests me. I really like the time element of it. Because you have to shoot the channels separately, you get this time-offset that can create ghostly artifacts. Take, for example, shooting a bridge. If in the red channel's exposure there's a person walking, but they're not there in the green or blue, you see this ghostly artifact of a thing. Or, better yet, if you manage to capture them in different parts of their journey in the channels. If they're walking left to right you might get a red image in the left, a green in the center, and a blue to the left. It's really quite charming.

So that's what I'm hoping to recreate here. `dslr-trichrome` expects three images, one for each channel.

Use the `-e` option to print the exif data of the selected images. This is particularly useful if you're shooting for the bracketed (`-k`) exposure scheme and accidentally exposed out-of-order (oops ^^;;;) so you can name the files accordingly.

You can specify `-d` with a directory and it'll look for `red`, `green`, and `blue` file stems (file name without the extension) and use those as their channels. If you want to specify the channels individually, you can use the `-r`, `-g`, `-b` flags. You may not use `-d` and the channel flags together, sorry. Maybe at one point in time, but not now.

The output file is `trichrome.png` for the default scheme and `bracketed.png` for the bracketed scheme. It just puts the file in the current directory.

#### Default scheme
This assumes the shots have the same exposure and treats it as such. It debayers the images together, taking from the respective images color channels, and then goes about business as normal. whitebalance, sRGB; the whole thing.

#### Bracketed scheme; `-k`
Hehehehehe. This assumes you exposed each channel to how the camera perceives them. I.E, no white balance is applied. Colours are *as shot*, except we gamma correct them.