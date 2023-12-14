use ark_bls12_381::fr::Fr;
use ark_bls12_381::Fq as F;
use ark_crypto_primitives::sponge::poseidon::{PoseidonConfig, PoseidonSponge};
use ark_crypto_primitives::sponge::{
    Absorb, AbsorbWithLength, CryptographicSponge, FieldBasedCryptographicSponge,
};
use ark_crypto_primitives::{absorb, collect_sponge_bytes, collect_sponge_field_elements};
use ark_ff::{One, PrimeField, UniformRand};
use ark_r1cs_std::boolean::Boolean;
use ark_r1cs_std::R1CSVar;
use ark_relations::r1cs::{ConstraintSystem, SynthesisError};
use ark_std::test_rng;

use crate::graph_checks::cmp;
use crate::graph_checks::Boolean2DArray;
use ark_r1cs_std::alloc::AllocVar;

//construct the hash of a boolean vector
// 1. Generate Params 2. Preprocess matrix 3. create sponge
pub fn hasher<const N: usize, ConstraintF: PrimeField>(
    adj_matrix: &Boolean2DArray<N, ConstraintF>,
) -> Result<Vec<Fr>, SynthesisError> {
    let preprocess = matrix_flattener(&adj_matrix).unwrap();
    let mut sponge = sponge_create::<Fr>(&preprocess).unwrap();
    let hash = squeeze_sponge(&mut sponge).unwrap();

    Ok(hash)
}

// getting the params and creating a new sponge object, absorbing a single boolean vector
pub fn sponge_create<ConstraintF: PrimeField>(
    input: &Vec<bool>,
) -> Result<PoseidonSponge<Fr>, SynthesisError> {
    let sponge_param = poseidon_parameters_for_test();
    // let elem = Fr::rand(&mut rng);
    let mut sponge1 = PoseidonSponge::<Fr>::new(&sponge_param);
    sponge1.absorb(input);

    Ok(sponge1)
}

//squeezes a single field element (hash) from an existing sponge with data
pub fn squeeze_sponge(sponge: &mut PoseidonSponge<Fr>) -> Result<Vec<Fr>, SynthesisError> {
    let squeeze = sponge.squeeze_native_field_elements(1);
    Ok(squeeze.to_vec())
}
// Takes in a 2D Boolean array (representing an adjacency matrix) and flattens it into a boolean vector
//TODO: Implement bit-packing
pub fn matrix_flattener<const N: usize, ConstraintF: PrimeField>(
    adj_matrix: &Boolean2DArray<N, ConstraintF>,
) -> Result<Vec<bool>, SynthesisError> {
    let mut flattened_matrix = Vec::new();
    for i in 0..N {
        for j in 0..N {
            let transacted = &adj_matrix.0[i][j]; // true if person i sent to person j
            if transacted.value()? == true {
                flattened_matrix.push(true);
            } else {
                flattened_matrix.push(false);
            }
        }
    }
    Ok(flattened_matrix)
}

/// Generate default parameters (bls381-fr-only) for alpha = 17, state-size = 8
pub(crate) fn poseidon_parameters_for_test<F: PrimeField>() -> PoseidonConfig<F> {
    let alpha = 17;
    let mds = vec![
        vec![
            F::from_str(
                "43228725308391137369947362226390319299014033584574058394339561338097152657858",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "20729134655727743386784826341366384914431326428651109729494295849276339718592",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "14275792724825301816674509766636153429127896752891673527373812580216824074377",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "3039440043015681380498693766234886011876841428799441709991632635031851609481",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "6678863357926068615342013496680930722082156498064457711885464611323928471101",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "37355038393562575053091209735467454314247378274125943833499651442997254948957",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "26481612700543967643159862864328231943993263806649000633819754663276818191580",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "30103264397473155564098369644643015994024192377175707604277831692111219371047",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "5712721806190262694719203887224391960978962995663881615739647362444059585747",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
    ];
    let ark = vec![
        vec![
            F::from_str(
                "44595993092652566245296379427906271087754779418564084732265552598173323099784",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "23298463296221002559050231199021122673158929708101049474262017406235785365706",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "34212491019164671611180318500074499609633402631511849759183986060951187784466",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "19098051134080182375553680073525644187968170656591203562523489333616681350367",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "7027675418691353855077049716619550622043312043660992344940177187528247727783",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "47642753235356257928619065424282314733361764347085604019867862722762702755609",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "24281836129477728386327945482863886685457469794572168729834072693507088619997",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "12624893078331920791384400430193929292743809612452779381349824703573823883410",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "22654862987689323504199204643771547606936339944127455903448909090318619188561",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "27229172992560143399715985732065737093562061782414043625359531774550940662372",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "13224952063922250960936823741448973692264041750100990569445192064567307041002",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "40380869235216625717296601204704413215735530626882135230693823362552484855508",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "4245751157938905689397184705633683893932492370323323780371834663438472308145",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "8252156875535418429533049587170755750275631534314711502253775796882240991261",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "32910829712934971129644416249914075073083903821282503505466324428991624789936",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "49412601297460128335642438246716127241669915737656789613664349252868389975962",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "841661305510340459373323516098909074520942972558284146843779636353111592117",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "37926489020263024391336570420006226544461516787280929232555625742588667303947",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "18433043696013996573551852847056868761017170818820490351056924728720017242180",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "45376910275288438312773930242803223482318753992595269901397542214841496212310",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "47854349410014339708332226068958253098964727682486278458389508597930796651514",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "32638426693771251366613055506166587312642876874690861030672730491779486904360",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "19105439281696418043426755774110765432959446684037017837894045255490581318047",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "13484299981373196201166722380389594773562113262309564134825386266765751213853",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "63360321133852659797114062808297090090814531427710842859827725871241144161",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "42427543035537409467993338717379268954936885184662765745740070438835506287271",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "149101987103211771991327927827692640556911620408176100290586418839323044234",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "8341764062226826803887898710015561861526081583071950015446833446251359696930",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "45635980415044299013530304465786867101223925975971912073759959440335364441441",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "49833261156201520743834327917353893365097424877680239796845398698940689734850",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "26764715016591436228000634284249890185894507497739511725029482580508707525029",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "25054530812095491217523557726611612265064441619646263299990388543372685322499",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "47654590955096246997622155031169641628093104787883934397920286718814889326452",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "16463825890556752307085325855351334996898686633642574805918056141310194135796",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "17473961341633494489168064889016732306117097771640351649096482400214968053040",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "49914603434867854893558366922996753035832008639512305549839666311012232077468",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "17122578514152308432111470949473865420090463026624297565504381163777697818362",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "34870689836420861427379101859113225049736283485335674111421609473028315711541",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "4622082908476410083286670201138165773322781640914243047922441301693321472984",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "6079244375752010013798561155333454682564824861645642293573415833483620500976",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "2635090520059500019661864086615522409798872905401305311748231832709078452746",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "19070766579582338321241892986615538320421651429118757507174186491084617237586",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "12622420533971517050761060317049369208980632120901481436392835424625664738526",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "8965101225657199137904506150282256568170501907667138404080397024857524386266",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "27085091008069524593196374148553176565775450537072498305327481366756159319838",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "45929056591150668409624595495643698205830429971690813312608217341940499221218",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "50361689160518167880500080025023064746137161030119436080957023803101861300846",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "6722586346537620732668048024627882970582133613352245923413730968378696371065",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "7340485916200743279276570085958556798507770452421357119145466906520506506342",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "25946733168219652706630789514519162148860502996914241011500280690204368174083",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "9962367658743163006517635070396368828381757404628822422306438427554934645464",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "7221669722700687417346373353960536661883467014204005276831020252277657076044",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "21487980358388383563030903293359140836304488103090321183948009095669344637431",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "44389482047246878765773958430749333249729101516826571588063797358040130313157",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "32887270862917330820874162842519225370447850172085449103568878409533683733185",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "15453393396765207016379045014101989306173462885430532298601655955681532648226",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "5478929644476681096437469958231489102974161353940993351588559414552523375472",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "41981370411247590312677561209178363054744730805951096631186178388981705304138",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "3474136981645476955784428843999869229067282976757744542648188369810577298585",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "26251477770740399889956219915654371915771248171098220204692699710414817081869",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "51916561889718854106125837319509539220778634838409949714061033196765117231752",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "25355145802812435959748831835587713214179184608408449220418373832038339021974",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "31950684570730625275416731570246297947385359051792335826965013637877068017530",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "40966378914980473680181850710703295982197782082391794594149984057481543436879",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "1141315130963422417761731263662398620858625339733452795772225916965481730059",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "9812100862165422922235757591915383485338044715409891361026651619010947646011",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "25276091996614379065765602410190790163396484122487585763380676888280427744737",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "18512694312063606403196469408971540495273694846641903978723927656359350642619",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "5791584766415439694303685437881192048262049244830616851865505314899699012588",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "34501536331706470927069149344450300773777486993504673779438188495686129846168",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "10797737565565774079718466476236831116206064650762676383469703413649447678207",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "42599392747310354323136214835734307933597896695637215127297036595538235868368",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "1336670998775417133322626564820911986969949054454812685145275612519924150700",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "2630141283339761901081411552890260088516693208402906795133548756078952896770",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "5206688943117414740600380377278238268309952400341418217132724749372435975215",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "10739264253827005683370721104077252560524362323422172665530191908848354339715",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "48010640624945719826344492755710886355389194986527731603685956726907395779674",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "47880724693177306044229143357252697148359033158394459365791331000715957339701",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "51658938856669444737833983076793759752280196674149218924101718974926964118996",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "27558055650076329657496888512074319504342606463881203707330358472954748913263",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "38886981777859313701520424626728402175860609948757992393598285291689196608037",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "17152756165118461969542990684402410297675979513690903033350206658079448802479",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "43766946932033687220387514221943418338304186408056458476301583041390483707207",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "24324495647041812436929170644873622904287038078113808264580396461953421400343",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "6935839211798937659784055008131602708847374430164859822530563797964932598700",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "42126767398190942911395299419182514513368023621144776598842282267908712110039",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "5702364486091252903915715761606014714345316580946072019346660327857498603375",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "28184981699552917714085740963279595942132561155181044254318202220270242523053",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "27078204494010940048327822707224393686245007379331357330801926151074766130790",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "5004172841233947987988267535285080365124079140142987718231874743202918551203",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "7974360962120296064882769128577382489451060235999590492215336103105134345602",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "48062035869818179910046292951628308709251170031813126950740044942870578526376",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "26361151154829600651603985995297072258262605598910254660032612019129606811983",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "46973867849986280770641828877435510444176572688208439836496241838832695841519",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "1219439673853113792340300173186247996249367102884530407862469123523013083971",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "8063356002935671186275773257019749639571745240775941450161086349727882957042",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "8815571992701260640209942886673939234666734294275300852283020522390608544536",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "36384568984671043678320545346945893232044626942887414733675890845013312931948",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "7493936589040764830842760521372106574503511314427857201860148571929278344956",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "26516538878265871822073279450474977673130300973488209984756372331392531193948",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "3872858659373466814413243601289105962248870842202907364656526273784217311104",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "8291822807524000248589997648893671538524566700364221355689839490238724479848",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "32842548776827046388198955038089826231531188946525483251252938248379132381248",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "10749428410907700061565796335489079278748501945557710351216806276547834974736",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "43342287917341177925402357903832370099402579088513884654598017447701677948416",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "29658571352070370791360499299098360881857072189358092237807807261478461425147",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "7805182565862454238315452208989152534554369855020544477885853141626690738363",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "30699555847500141715826240743138908521140760599479365867708690318477369178275",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
        vec![
            F::from_str(
                "1231951350103545216624376889222508148537733140742167414518514908719103925687",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "24784260089125933876714702247471508077514206350883487938806451152907502751770",
            )
            .map_err(|_| ())
            .unwrap(),
            F::from_str(
                "36563542611079418454711392295126742705798573252480028863133394504154697924536",
            )
            .map_err(|_| ())
            .unwrap(),
        ],
    ];
    let full_rounds = 8;
    let total_rounds = 37;
    let partial_rounds = total_rounds - full_rounds;
    let capacity = 1;
    let rate = 2;
    PoseidonConfig {
        full_rounds,
        partial_rounds,
        alpha,
        ark,
        mds,
        rate,
        capacity,
    }
}

#[test]
fn mod_gen_hash_test() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;

    let adj_matrix = [
        [false, true, true, false],   //               [0]
        [false, false, true, false],  //               / \
        [false, false, false, true],  //             [1]->[2] -> 3
        [false, false, false, false], //
    ];

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();

    let hash1 = hasher(&adj_matrix_var).unwrap();
    let hash2 = hasher(&adj_matrix_var).unwrap();

    // Check if hashes are consistent for the same input
    assert_eq!(hash1, hash2);

    // Modify the adjacency matrix
    let adj_matrix_modified = [
        [true, true, false, false],   //              [0]
        [false, false, true, false],  //              /  \
        [false, false, false, true],  //             [1]->[2] -> 3
        [false, false, false, false], //
    ];
    let adj_matrix_var_modified =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_modified)).unwrap();
    let hash_modified = hasher(&adj_matrix_var_modified).unwrap();

    // Check if hash changes with different input
    assert_ne!(hash1, hash_modified);
}

#[test]
fn test_hashing_empty_matrix() {
    use ark_bls12_381::Fq as F;
    let adj_matrix = [[false; 4]; 4];

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let hash = hasher(&adj_matrix_var).unwrap();

    // Ensure hash is not empty or null
    assert!(!hash.is_empty());
}

#[test]
fn test_hashing_full_matrix() {
    use ark_bls12_381::Fq as F;
    let adj_matrix = [[true; 4]; 4];

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let hash = hasher(&adj_matrix_var).unwrap();

    // Assert the hash is generated successfully
    assert!(!hash.is_empty());
}

#[test]
fn test_hashing_different_matrices() {
    use ark_bls12_381::Fq as F;
    let adj_matrix_1 = [[false, true], [true, false]];
    let adj_matrix_2 = [[true, false], [false, true]];

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var_1 = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_1)).unwrap();
    let adj_matrix_var_2 = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_2)).unwrap();

    let hash1 = hasher(&adj_matrix_var_1).unwrap();
    let hash2 = hasher(&adj_matrix_var_2).unwrap();

    assert_ne!(hash1, hash2);
}

#[test]
fn test_hashing_one_changed_element() {
    use ark_bls12_381::Fq as F;
    let adj_matrix_1 = [[false; 3]; 3];
    let mut adj_matrix_2 = adj_matrix_1.clone();
    adj_matrix_2[1][1] = true; // Change one element

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var_1 = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_1)).unwrap();
    let adj_matrix_var_2 = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_2)).unwrap();

    let hash1 = hasher(&adj_matrix_var_1).unwrap();
    let hash2 = hasher(&adj_matrix_var_2).unwrap();

    assert_ne!(hash1, hash2);
}

#[test]
fn test_hashing_inverted_matrices() {
    use ark_bls12_381::Fq as F;
    let adj_matrix = [[true, false], [false, true]];
    let inverted_matrix = adj_matrix.map(|row| row.map(|elem| !elem));

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let inverted_matrix_var =
        Boolean2DArray::new_witness(cs.clone(), || Ok(inverted_matrix)).unwrap();

    let hash1 = hasher(&adj_matrix_var).unwrap();
    let hash2 = hasher(&inverted_matrix_var).unwrap();

    assert_ne!(hash1, hash2);
}

#[test]
fn test_hashing_large_identical_matrices() {
    use ark_bls12_381::Fq as F;
    const N: usize = 100; // Large size
    let mut adj_matrix_1 = [[false; N]; N];
    let mut adj_matrix_2 = [[false; N]; N];

    // Initialize both matrices with the same pattern
    for i in 0..N {
        for j in 0..N {
            if i % 2 == 0 && j % 3 == 0 {
                adj_matrix_1[i][j] = true;
                adj_matrix_2[i][j] = true;
            }
        }
    }

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var_1 = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_1)).unwrap();
    let adj_matrix_var_2 = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_2)).unwrap();

    let hash1 = hasher(&adj_matrix_var_1).unwrap();
    let hash2 = hasher(&adj_matrix_var_2).unwrap();

    assert_eq!(hash1, hash2);
}

#[test]
fn test_hashing_large_diagonal_matrices() {
    use ark_bls12_381::Fq as F;
    const N: usize = 50; // Large size
    let mut adj_matrix = [[false; N]; N];

    // Diagonal true values
    for i in 0..N {
        adj_matrix[i][i] = true;
    }

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var_1 =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix.clone())).unwrap();
    let adj_matrix_var_2 =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix.clone())).unwrap();

    let hash1 = hasher(&adj_matrix_var_1).unwrap();
    let hash2 = hasher(&adj_matrix_var_2).unwrap();

    assert_eq!(hash1, hash2);
}

#[test]
fn test_hashing_large_sparse_matrices() {
    use ark_bls12_381::Fq as F;
    const N: usize = 60; // Large size
    let mut adj_matrix = [[false; N]; N];

    // Sparse true values
    for i in (0..N).step_by(10) {
        for j in (0..N).step_by(15) {
            adj_matrix[i][j] = true;
        }
    }

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var_1 =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix.clone())).unwrap();
    let adj_matrix_var_2 =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix.clone())).unwrap();

    let hash1 = hasher(&adj_matrix_var_1).unwrap();
    let hash2 = hasher(&adj_matrix_var_2).unwrap();

    assert_eq!(hash1, hash2);
}
