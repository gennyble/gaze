I cannot afford to shoot film nor can I afford the three filters I'd need to shoot trichrome at all, but it interests me. I really like the time element of it. Because you have to shoot the channels separately, you get this time-offset that can create ghostly artifacts. Take, for example, shooting a bridge. If in the red channel's exposure there's a person walking, but they're not there in the green or blue, you see this ghostly artifact of a thing. Or, better yet, if you manage to capture them in different parts of their journey in the channels. If they're walking left to right you might get a red image in the left, a green in the center, and a blue to the left. It's really quite charming.

So that's what I'm hoping to recreate here. `dslr-trichrome` expects three images, one for each channel.

Use the `-e` option to print the exif data of the selected images. This is particularly useful if you're shooting for the bracketed (`-k`) exposure scheme and accidentally exposed out-of-order (oops ^^;;;). 

#### Bracketed scheme; `-k`
Hehehehehe. This assumes you exposed each channel to how the camera perceives them. I.E, no white balance is applied. Colours are *as shot*, except we gamma correct them.