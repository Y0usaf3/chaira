# goated script made by @Vimthusiast

function imgsplit
    if test (count $argv) -lt 3
        echo "Usage: imgsplit <image> <cols> <rows> [output_dir]"
        echo ""
        echo "Default output directory:"
        echo "  <image>.split/"
        return 1
    end

    set input $argv[1]
    set cols $argv[2]
    set rows $argv[3]

    set full_name (basename $input)
    set clean_name (string replace -r '\.[^.]*$' '' $full_name)

    set default_outdir "$full_name.split"

    if test (count $argv) -ge 4
        set outdir $argv[4]
    else
        set outdir $default_outdir
    end

    mkdir -p $outdir

    set width (ffprobe -v error -select_streams v:0 -show_entries stream=width -of csv=p=0 $input)
    set height (ffprobe -v error -select_streams v:0 -show_entries stream=height -of csv=p=0 $input)

    set tile_w (math "$width / $cols")
    set tile_h (math "$height / $rows")

    set total (math "$cols * $rows")
    set idx 0

    for y in (seq 0 (math "$rows - 1"))
        for x in (seq 0 (math "$cols - 1"))

            set idx (math "$idx + 1")
            set out "$outdir/$clean_name"_"$y"_"$x.gif"

            echo "$idx/$total -> $clean_name"_"$y"_"$x.gif"

            ffmpeg -v error -y -i $input -vf \
            "crop=$tile_w:$tile_h:(in_w/$cols)*$x:(in_h/$rows)*$y" \
            -f gif $out

        end
    end
end
